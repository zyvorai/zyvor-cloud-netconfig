// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const EC2_METADATA_ENDPOINT: &str = "http://169.254.169.254/latest/meta-data";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EC2System {
    pub instance_id: String,
    pub instance_type: String,
    pub local_ipv4: String,
    pub public_ipv4: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EC2MacData {
    pub mac: String,
    pub local_ipv4s: Vec<String>,
    pub subnet_ipv4_cidr_block: String,
}

#[derive(Clone)]
pub struct EC2 {
    system: HashMap<String, serde_json::Value>,
    #[allow(dead_code)] // captured metadata, not yet consumed
    network: HashMap<String, serde_json::Value>,
    macs: HashMap<String, EC2MacData>,
    #[allow(dead_code)] // captured metadata, not yet consumed
    imds_version: u8,
}

impl EC2 {
    pub fn new(config: &crate::conf::AwsCloudConfig) -> Self {
        Self {
            system: HashMap::new(),
            network: HashMap::new(),
            macs: HashMap::new(),
            imds_version: config.imds_version,
        }
    }

    async fn fetch_metadata_simple(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", EC2_METADATA_ENDPOINT, path);
        let client = reqwest::Client::new();
        let text = client.get(&url).send().await?.text().await?;
        Ok(text)
    }

    fn parse_ipv4_addresses_from_metadata(
        &self,
        addresses: &str,
        cidr: &str,
    ) -> HashMap<String, bool> {
        let mut result = HashMap::new();
        let prefix = cidr.split('/').nth(1).unwrap_or("24");

        for addr in addresses.split(',') {
            let addr = addr.trim();
            if !addr.is_empty() {
                result.insert(format!("{}/{}", addr, prefix), true);
            }
        }

        result
    }
}

pub(super) async fn ec2_configure_network(ec2: &EC2, env: &mut super::Environment) -> Result<()> {
    let macs: Vec<String> = ec2.macs.keys().cloned().collect();
    for mac in macs {
        let Some(mac_data) = ec2.macs.get(&mac) else {
            continue;
        };
        let Some(link) = env.links.links_by_mac.get(&mac).cloned() else {
            continue;
        };
        let addresses_str = mac_data.local_ipv4s.join(",");
        let addresses = ec2
            .parse_ipv4_addresses_from_metadata(&addresses_str, &mac_data.subnet_ipv4_cidr_block);

        if !addresses.is_empty() {
            super::network::configure_network(env, &link, addresses, None, None).await?;
        }
    }
    Ok(())
}

#[async_trait::async_trait]
impl super::CloudProvider for EC2 {
    async fn fetch_cloud_metadata(&mut self) -> Result<()> {
        // Simplified EC2 metadata fetching
        // In a full implementation, this would recursively traverse the metadata tree

        // Fetch MACs
        let macs_text = self
            .fetch_metadata_simple("network/interfaces/macs/")
            .await?;
        let macs: Vec<String> = macs_text
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.trim_end_matches('/').to_string())
            .collect();

        // Fetch data for each MAC
        for mac in macs {
            let local_ipv4s = self
                .fetch_metadata_simple(&format!("network/interfaces/macs/{}/local-ipv4s", mac))
                .await
                .unwrap_or_default();

            let subnet_cidr = self
                .fetch_metadata_simple(&format!(
                    "network/interfaces/macs/{}/subnet-ipv4-cidr-block",
                    mac
                ))
                .await
                .unwrap_or_else(|_| "10.0.0.0/24".to_string());

            let mac_data = EC2MacData {
                mac: mac.clone(),
                local_ipv4s: local_ipv4s.lines().map(|s| s.to_string()).collect(),
                subnet_ipv4_cidr_block: subnet_cidr,
            };

            self.macs.insert(mac, mac_data);
        }

        Ok(())
    }

    async fn configure_network_from_cloud_meta(&self, env: &mut super::Environment) -> Result<()> {
        ec2_configure_network(self, env).await
    }

    async fn save_cloud_metadata(&self) -> Result<()> {
        let path = format!("{}/ec2", crate::conf::SYSTEM_STATE_DIR);
        crate::system::create_and_save_json(&path, &self.system)?;
        Ok(())
    }

    async fn link_save_cloud_metadata(&self, env: &super::Environment) -> Result<()> {
        for (mac, mac_data) in &self.macs {
            if let Some(link) = env.links.links_by_mac.get(mac) {
                let path = format!("{}/{}", crate::conf::LINK_STATE_DIR, link.name);
                crate::system::create_and_save_json(&path, mac_data)?;
            }
        }
        Ok(())
    }
}
