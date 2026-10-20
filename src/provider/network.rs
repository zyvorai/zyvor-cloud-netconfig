// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use crate::network::{self, Link, Route, RoutingPolicyRule};
use anyhow::Result;
use std::collections::HashMap;

pub async fn configure_network(
    env: &mut super::Environment,
    link: &Link,
    new_addresses: HashMap<String, bool>,
    gateway: Option<String>,
    mtu: Option<u32>,
) -> Result<()> {
    tracing::info!(
        "Link='{}' ifindex='{}' configuring network ...",
        link.name,
        link.ifindex
    );

    // Bring link up if needed
    if link.oper_state != "Up" {
        network::link_set_oper_state_up(link.ifindex).await?;
    }

    // Set MTU if different
    if let Some(new_mtu) = mtu {
        if link.mtu != new_mtu {
            network::link_set_mtu(link.ifindex, new_mtu).await?;
        }
    }

    // Configure addresses
    for addr in new_addresses.keys() {
        network::address_set(&link.name, addr).await?;
        tracing::info!(
            "Successfully added address='{}' on link='{}' ifindex='{}'",
            addr,
            link.name,
            link.ifindex
        );
    }

    // Configure route
    configure_route(env, link, gateway.as_deref()).await?;

    // Configure routing policy rules for each address
    for addr in new_addresses.keys() {
        configure_routing_policy_rule(env, link, addr).await?;
    }

    // Remove old addresses that are no longer in metadata
    let stale_addrs: Vec<String> = env
        .addresses_by_mac
        .get(&link.mac)
        .map(|old_addrs| {
            old_addrs
                .keys()
                .filter(|addr| !new_addresses.contains_key(*addr))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    for old_addr in stale_addrs {
        remove_routing_policy_rule(env, &old_addr, link).await?;
        network::address_remove(&link.name, &old_addr).await?;
        tracing::info!(
            "Removed address='{}' from link='{}' ifindex='{}'",
            old_addr,
            link.name,
            link.ifindex
        );
    }

    // Update environment state
    env.addresses_by_mac.insert(link.mac.clone(), new_addresses);

    Ok(())
}

async fn configure_route(
    env: &mut super::Environment,
    link: &Link,
    gateway: Option<&str>,
) -> Result<()> {
    let gw = match gateway {
        Some(gw_str) => gw_str.to_string(),
        None => network::get_ipv4_gateway(link.ifindex).await?,
    };

    let table = env.route_table + link.ifindex;

    let route = Route {
        table,
        if_index: link.ifindex,
        gw: gw.clone(),
    };

    network::route_add(&route).await?;
    env.routes_by_index.insert(link.ifindex, route);

    tracing::info!(
        "Successfully added default gateway='{}' for link='{}' ifindex='{}' table='{}'",
        gw,
        link.name,
        link.ifindex,
        table
    );

    Ok(())
}

async fn configure_routing_policy_rule(
    env: &mut super::Environment,
    link: &Link,
    address: &str,
) -> Result<()> {
    let ip_str = address.split('/').next().unwrap_or(address);
    let table = env.route_table + link.ifindex;

    // Add "from" rule
    let from_rule = RoutingPolicyRule {
        from: Some(ip_str.to_string()),
        to: None,
        table,
    };

    network::routing_policy_rule_add(&from_rule).await?;
    env.routing_rules_by_address_from
        .insert(ip_str.to_string(), from_rule.clone());

    tracing::info!(
        "Successfully added routing policy rule 'from' in route table='{}' for link='{}' ifindex='{}'",
        table, link.name, link.ifindex
    );

    // Add "to" rule
    let to_rule = RoutingPolicyRule {
        from: None,
        to: Some(ip_str.to_string()),
        table,
    };

    network::routing_policy_rule_add(&to_rule).await?;
    env.routing_rules_by_address_to
        .insert(ip_str.to_string(), to_rule);

    tracing::info!(
        "Successfully added routing policy rule 'to' in route table='{}' for link='{}' ifindex='{}'",
        table, link.name, link.ifindex
    );

    Ok(())
}

async fn remove_routing_policy_rule(
    env: &mut super::Environment,
    address: &str,
    link: &Link,
) -> Result<()> {
    let ip_str = address.split('/').next().unwrap_or(address);

    // Remove "from" rule
    if let Some(rule) = env.routing_rules_by_address_from.remove(ip_str) {
        network::routing_policy_rule_remove(&rule).await?;
    }

    // Remove "to" rule
    if let Some(rule) = env.routing_rules_by_address_to.remove(ip_str) {
        network::routing_policy_rule_remove(&rule).await?;
    }

    // Remove route if no more rules for this link
    let table = env.route_table + link.ifindex;
    if is_rules_by_table_empty(env, table) {
        if let Some(route) = env.routes_by_index.remove(&link.ifindex) {
            network::route_remove(&route).await?;
        }
    }

    Ok(())
}

fn is_rules_by_table_empty(env: &super::Environment, table: u32) -> bool {
    let has_from = env
        .routing_rules_by_address_from
        .values()
        .any(|rule| rule.table == table);

    let has_to = env
        .routing_rules_by_address_to
        .values()
        .any(|rule| rule.table == table);

    !has_from && !has_to
}
