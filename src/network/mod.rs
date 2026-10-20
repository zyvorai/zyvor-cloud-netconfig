// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

mod address;
mod link;
mod route;
mod routing_policy_rule;

pub use address::*;
pub use link::*;
pub use route::*;
pub use routing_policy_rule::*;

use anyhow::Result;

pub async fn configure_by_index(if_index: u32) -> Result<()> {
    let gw = get_ipv4_gateway(if_index).await?;

    let route = Route {
        table: crate::conf::ROUTE_TABLE_BASE + if_index,
        if_index,
        gw: gw.clone(),
    };

    route_add(&route).await?;

    let link_name = get_link_name_by_index(if_index).await?;
    let addresses = get_ipv4_addresses(&link_name).await?;

    for addr in addresses.keys() {
        // Extract IP without prefix
        let ip_str = addr.split('/').next().unwrap_or(addr);

        let from = RoutingPolicyRule {
            from: Some(ip_str.to_string()),
            to: None,
            table: crate::conf::ROUTE_TABLE_BASE + if_index,
        };

        routing_policy_rule_add(&from).await?;

        let to = RoutingPolicyRule {
            from: None,
            to: Some(ip_str.to_string()),
            table: crate::conf::ROUTE_TABLE_BASE + if_index,
        };

        routing_policy_rule_add(&to).await?;
    }

    Ok(())
}

pub async fn configure_supplementary_links(supplementary: &str) -> Result<()> {
    let words: Vec<&str> = supplementary.split_whitespace().collect();

    if words.is_empty() {
        return Ok(());
    }

    for name in words {
        match get_link_index_by_name(name).await {
            Ok(index) => {
                if let Err(e) = configure_by_index(index).await {
                    tracing::error!(
                        "Failed to configure network for link='{}' ifindex='{}': {}",
                        name,
                        index,
                        e
                    );
                    return Err(e);
                }
                tracing::debug!(
                    "Successfully configured network for link='{}' ifindex='{}'",
                    name,
                    index
                );
            }
            Err(e) => {
                tracing::debug!("Failed to find link='{}'. Ignoring...: {}", name, e);
                continue;
            }
        }
    }

    Ok(())
}

pub async fn get_ipv4_gateway(if_index: u32) -> Result<String> {
    // Try to get default gateway by link
    if let Ok(gw) = get_default_ipv4_gateway_by_link(if_index).await {
        return Ok(gw);
    }

    // Try to get any gateway by link
    if let Ok(gw) = get_ipv4_gateway_by_link(if_index).await {
        return Ok(gw);
    }

    // Fall back to system default gateway
    get_default_ipv4_gateway().await
}
