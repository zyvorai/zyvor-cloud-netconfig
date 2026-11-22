// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-2.0

mod azure;
mod ec2;
mod gcp;
mod network;
mod watch;

pub use azure::*;
pub use ec2::*;
pub use gcp::*;
pub use network::*;
pub use watch::*;

use crate::cloud::CloudProvider as CloudKind;
use crate::network::{Links, Route, RoutingPolicyRule};
use anyhow::Result;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait CloudProvider: Send + Sync {
    async fn fetch_cloud_metadata(&mut self) -> Result<()>;
    async fn configure_network_from_cloud_meta(&self, env: &mut Environment) -> Result<()>;
    async fn save_cloud_metadata(&self) -> Result<()>;
    async fn link_save_cloud_metadata(&self, env: &Environment) -> Result<()>;
}

#[derive(Clone)]
pub enum Provider {
    Azure(Azure),
    Aws(EC2),
    Gcp(GCP),
}

impl Provider {
    async fn fetch_cloud_metadata(&mut self) -> Result<()> {
        match self {
            Provider::Azure(p) => p.fetch_cloud_metadata().await,
            Provider::Aws(p) => p.fetch_cloud_metadata().await,
            Provider::Gcp(p) => p.fetch_cloud_metadata().await,
        }
    }

    async fn save_cloud_metadata(&self) -> Result<()> {
        match self {
            Provider::Azure(p) => p.save_cloud_metadata().await,
            Provider::Aws(p) => p.save_cloud_metadata().await,
            Provider::Gcp(p) => p.save_cloud_metadata().await,
        }
    }

    async fn link_save_cloud_metadata(&self, env: &Environment) -> Result<()> {
        match self {
            Provider::Azure(p) => p.link_save_cloud_metadata(env).await,
            Provider::Aws(p) => p.link_save_cloud_metadata(env).await,
            Provider::Gcp(p) => p.link_save_cloud_metadata(env).await,
        }
    }
}

pub struct Environment {
    pub kind: CloudKind,
    pub provider: Provider,
    pub links: Links,
    pub route_table: u32,
    pub addresses_by_mac: HashMap<String, HashMap<String, bool>>,
    pub routes_by_index: HashMap<u32, Route>,
    pub routing_rules_by_address_from: HashMap<String, RoutingPolicyRule>,
    pub routing_rules_by_address_to: HashMap<String, RoutingPolicyRule>,
}

impl Environment {
    pub fn new(kind: CloudKind, config: &crate::conf::Config) -> Option<Self> {
        let provider = match kind {
            CloudKind::Azure => Provider::Azure(Azure::new(&config.cloud.azure)),
            CloudKind::AWS => Provider::Aws(EC2::new(&config.cloud.aws)),
            CloudKind::GCP => Provider::Gcp(GCP::new(&config.cloud.gcp)),
            _ => return None,
        };

        Some(Self {
            kind,
            provider,
            links: Links::new(),
            route_table: config.network.routing.table_base,
            addresses_by_mac: HashMap::new(),
            routes_by_index: HashMap::new(),
            routing_rules_by_address_from: HashMap::new(),
            routing_rules_by_address_to: HashMap::new(),
        })
    }
}

pub async fn acquire_cloud_metadata(env: &mut Environment) -> Result<()> {
    env.links = crate::network::acquire_links().await?;
    env.provider.fetch_cloud_metadata().await?;

    Ok(())
}

pub async fn configure_network_metadata(env: &mut Environment) -> Result<()> {
    // Cloned so the match doesn't hold env.provider borrowed while the arms
    // also need &mut env (env.provider aliases the whole env otherwise).
    let provider = env.provider.clone();
    match provider {
        Provider::Azure(azure) => azure_configure_network(&azure, env).await,
        Provider::Aws(ec2) => ec2_configure_network(&ec2, env).await,
        Provider::Gcp(gcp) => gcp_configure_network(&gcp, env).await,
    }
}

pub async fn save_metadata(env: &Environment) -> Result<()> {
    env.provider.save_cloud_metadata().await?;
    env.provider.link_save_cloud_metadata(env).await?;
    Ok(())
}
