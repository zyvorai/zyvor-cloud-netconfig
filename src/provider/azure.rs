// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const AZURE_METADATA_BASE: &str = "http://169.254.169.254/metadata/instance";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureMetadata {
    pub compute: AzureCompute,
    pub network: AzureNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureCompute {
    pub name: String,
    pub location: String,
    pub vm_id: String,
    pub vm_size: String,
    #[serde(default)]
    pub zone: String,
    #[serde(default)]
    pub subscription_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureNetwork {
    pub interface: Vec<AzureInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureInterface {
    pub mac_address: String,
    pub ipv4: AzureIpv4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureIpv4 {
    pub ip_address: Vec<AzureIpAddress>,
    pub subnet: Vec<AzureSubnet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureIpAddress {
    pub private_ip_address: String,
    pub public_ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSubnet {
    pub address: String,
    pub prefix: String,
}

#[derive(Clone)]
pub struct Azure {
    metadata: Option<AzureMetadata>,
    api_version: String,
}

impl Azure {
    pub fn new(config: &crate::conf::AzureCloudConfig) -> Self {
        Self {
            metadata: None,
            api_version: config.api_version.clone(),
        }
    }

    fn parse_ipv4_addresses_from_metadata_by_mac(&self, mac: &str) -> HashMap<String, bool> {
        let mut addresses = HashMap::new();

        if let Some(ref meta) = self.metadata {
            for iface in &meta.network.interface {
                if crate::parser::normalize_mac(&iface.mac_address)
                    == crate::parser::normalize_mac(mac)
                {
                    if let Some(subnet) = iface.ipv4.subnet.first() {
                        let prefix = &subnet.prefix;
                        for ip_addr in &iface.ipv4.ip_address {
                            let cidr = format!("{}/{}", ip_addr.private_ip_address, prefix);
                            addresses.insert(cidr, true);
                        }
                    }
                    break;
                }
            }
        }

        addresses
    }
}

pub(super) async fn azure_configure_network(
    azure: &Azure,
    env: &mut super::Environment,
) -> Result<()> {
    let macs: Vec<String> = env.links.links_by_mac.keys().cloned().collect();
    for mac in macs {
        let addresses = azure.parse_ipv4_addresses_from_metadata_by_mac(&mac);
        if addresses.is_empty() {
            continue;
        }
        let Some(link) = env.links.links_by_mac.get(&mac).cloned() else {
            continue;
        };
        super::network::configure_network(env, &link, addresses, None, None).await?;
    }
    Ok(())
}

#[async_trait::async_trait]
impl super::CloudProvider for Azure {
    async fn fetch_cloud_metadata(&mut self) -> Result<()> {
        let url = format!("{}?api-version={}", AZURE_METADATA_BASE, self.api_version);
        let client = reqwest::Client::new();
        let response = client.get(&url).header("Metadata", "true").send().await?;

        self.metadata = Some(response.json::<AzureMetadata>().await?);
        Ok(())
    }

    async fn configure_network_from_cloud_meta(&self, env: &mut super::Environment) -> Result<()> {
        azure_configure_network(self, env).await
    }

    async fn save_cloud_metadata(&self) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            let path = format!("{}/azure", crate::conf::SYSTEM_STATE_DIR);
            crate::system::create_and_save_json(&path, &meta.compute)?;
        }
        Ok(())
    }

    async fn link_save_cloud_metadata(&self, env: &super::Environment) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.network.interface {
                let mac = crate::parser::normalize_mac(&iface.mac_address);
                if let Some(link) = env.links.links_by_mac.get(&mac) {
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

    // Shape and macAddress format taken from the Azure IMDS documentation
    // ("Sample 6: Retrieve network information"): bare, upper-case hex.
    const IMDS_SAMPLE: &str = r#"{
        "compute": {
            "name": "examplevmname",
            "location": "westus",
            "vmId": "02aab8a4-74ef-476e-8182-f6d2ba4166a6",
            "vmSize": "Standard_A3"
        },
        "network": {
            "interface": [{
                "ipv4": {
                    "ipAddress": [{
                        "privateIpAddress": "10.144.133.132",
                        "publicIpAddress": ""
                    }],
                    "subnet": [{
                        "address": "10.144.133.128",
                        "prefix": "26"
                    }]
                },
                "macAddress": "000D3AF806EC"
            }]
        }
    }"#;

    fn azure_with_sample() -> Azure {
        Azure {
            metadata: Some(serde_json::from_str(IMDS_SAMPLE).unwrap()),
            api_version: "2021-02-01".to_string(),
        }
    }

    #[test]
    fn matches_link_mac_against_imds_mac() {
        // Links are keyed by the kernel's lower-case, colon-separated MAC.
        let addresses =
            azure_with_sample().parse_ipv4_addresses_from_metadata_by_mac("00:0d:3a:f8:06:ec");
        assert_eq!(addresses.len(), 1);
        assert!(addresses.contains_key("10.144.133.132/26"));
    }

    #[test]
    fn unknown_mac_yields_no_addresses() {
        let addresses =
            azure_with_sample().parse_ipv4_addresses_from_metadata_by_mac("00:0d:3a:00:00:01");
        assert!(addresses.is_empty());
    }
}
