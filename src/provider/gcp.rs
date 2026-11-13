// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GCP_METADATA_ENDPOINT: &str =
    "http://metadata.google.internal/computeMetadata/v1/?recursive=true";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCPMetadata {
    pub instance: GCPInstance,
    pub project: GCPProject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GCPInstance {
    /// The metadata server returns the instance ID as a JSON number.
    pub id: u64,
    pub hostname: String,
    pub machine_type: String,
    pub network_interfaces: Vec<GCPNetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GCPNetworkInterface {
    pub mac: String,
    pub ip: String,
    pub subnetmask: String,
    pub gateway: String,
    pub mtu: u32,
    #[serde(default)]
    pub ip_aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GCPProject {
    pub project_id: String,
    pub numeric_project_id: i64,
}

#[derive(Clone)]
pub struct GCP {
    metadata: Option<GCPMetadata>,
    #[allow(dead_code)] // captured metadata, not yet consumed
    recursive: bool,
}

impl GCP {
    pub fn new(config: &crate::conf::GcpCloudConfig) -> Self {
        Self {
            metadata: None,
            recursive: config.recursive,
        }
    }

    fn parse_ipv4_gateway_from_metadata_by_mac(&self, mac: &str) -> Option<String> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if iface.mac.eq_ignore_ascii_case(mac) {
                    return Some(iface.gateway.clone());
                }
            }
        }
        None
    }

    fn parse_link_mtu_from_metadata_by_mac(&self, mac: &str) -> Option<u32> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if iface.mac.eq_ignore_ascii_case(mac) {
                    return Some(iface.mtu);
                }
            }
        }
        None
    }

    fn parse_ipv4_addresses_from_metadata_by_mac(&self, mac: &str) -> HashMap<String, bool> {
        let mut addresses = HashMap::new();

        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if iface.mac.eq_ignore_ascii_case(mac) {
                    // Convert subnet mask to CIDR prefix
                    let prefix = self.subnet_mask_to_cidr(&iface.subnetmask);

                    // Add primary IP
                    let cidr = format!("{}/{}", iface.ip, prefix);
                    addresses.insert(cidr, true);

                    // Add single-IP aliases. The metadata server returns aliases
                    // in CIDR form ("10.128.2.5/32", "10.128.1.0/24"). A range is
                    // routed to the VM, not assigned to its NIC: adding it as an
                    // address would install an on-link route for the whole range
                    // and clash with the containers or services that use it.
                    for alias in &iface.ip_aliases {
                        match alias.split_once('/') {
                            None => {
                                addresses.insert(format!("{}/32", alias), true);
                            }
                            Some((_, "32")) => {
                                addresses.insert(alias.clone(), true);
                            }
                            Some(_) => tracing::debug!(
                                "Skipping alias IP range '{}': ranges are routed, not assigned",
                                alias
                            ),
                        }
                    }
                    break;
                }
            }
        }

        addresses
    }

    fn subnet_mask_to_cidr(&self, mask: &str) -> u8 {
        // Simple conversion from subnet mask to CIDR prefix length
        let parts: Vec<u8> = mask.split('.').filter_map(|s| s.parse().ok()).collect();
        if parts.len() != 4 {
            return 24; // default
        }

        let mut prefix = 0;
        for octet in parts {
            prefix += octet.count_ones();
        }
        prefix as u8
    }
}

pub(super) async fn gcp_configure_network(gcp: &GCP, env: &mut super::Environment) -> Result<()> {
    let macs: Vec<String> = env.links.links_by_mac.keys().cloned().collect();
    for mac in macs {
        let addresses = gcp.parse_ipv4_addresses_from_metadata_by_mac(&mac);
        if addresses.is_empty() {
            continue;
        }
        let Some(link) = env.links.links_by_mac.get(&mac).cloned() else {
            continue;
        };
        let gateway = gcp.parse_ipv4_gateway_from_metadata_by_mac(&mac);
        let mtu = gcp.parse_link_mtu_from_metadata_by_mac(&mac);

        super::network::configure_network(env, &link, addresses, gateway, mtu).await?;
    }
    Ok(())
}

#[async_trait::async_trait]
impl super::CloudProvider for GCP {
    async fn fetch_cloud_metadata(&mut self) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .get(GCP_METADATA_ENDPOINT)
            .header("Metadata-Flavor", "Google")
            .send()
            .await?;

        self.metadata = Some(response.json::<GCPMetadata>().await?);
        Ok(())
    }

    async fn configure_network_from_cloud_meta(&self, env: &mut super::Environment) -> Result<()> {
        gcp_configure_network(self, env).await
    }

    async fn save_cloud_metadata(&self) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            let path = format!("{}/gcp", crate::conf::SYSTEM_STATE_DIR);
            crate::system::create_and_save_json(&path, &meta.instance)?;
        }
        Ok(())
    }

    async fn link_save_cloud_metadata(&self, env: &super::Environment) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if let Some(link) = env.links.links_by_mac.get(&iface.mac) {
                    let path = format!("{}/{}", crate::conf::LINK_STATE_DIR, link.name);
                    crate::system::create_and_save_json(&path, iface)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape of `computeMetadata/v1/?recursive=true`: the instance `id` is a
    // JSON number and alias ranges are under `ipAliases`.
    const METADATA_SAMPLE: &str = r#"{
        "instance": {
            "id": 5775171277418378000,
            "hostname": "vm.c.example.internal",
            "machineType": "projects/123/machineTypes/e2-small",
            "networkInterfaces": [{
                "mac": "42:01:0a:80:00:02",
                "ip": "10.128.0.2",
                "subnetmask": "255.255.240.0",
                "gateway": "10.128.0.1",
                "mtu": 1460,
                "ipAliases": ["10.128.1.0/24", "10.128.2.5", "10.128.3.7/32"]
            }]
        },
        "project": {
            "projectId": "example",
            "numericProjectId": 708288290784
        }
    }"#;

    fn gcp_with_sample() -> GCP {
        let mut gcp = GCP::new(&crate::conf::GcpCloudConfig::default());
        gcp.metadata = Some(serde_json::from_str(METADATA_SAMPLE).unwrap());
        gcp
    }

    #[test]
    fn parses_recursive_metadata() {
        let meta: GCPMetadata = serde_json::from_str(METADATA_SAMPLE).unwrap();
        assert_eq!(meta.instance.id, 5775171277418378000);
        assert_eq!(
            meta.instance.network_interfaces[0].ip_aliases,
            vec!["10.128.1.0/24", "10.128.2.5", "10.128.3.7/32"]
        );
    }

    #[test]
    fn includes_primary_ip_and_single_ip_aliases_only() {
        let addresses =
            gcp_with_sample().parse_ipv4_addresses_from_metadata_by_mac("42:01:0a:80:00:02");
        assert!(addresses.contains_key("10.128.0.2/20"));
        assert!(addresses.contains_key("10.128.2.5/32"));
        assert!(addresses.contains_key("10.128.3.7/32"));
        assert!(!addresses.contains_key("10.128.1.0/24"));
        assert_eq!(addresses.len(), 3);
    }
}
