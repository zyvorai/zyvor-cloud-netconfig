// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use rtnetlink::packet_route::rule::{RuleAction, RuleAttribute, RuleMessage};
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct RoutingPolicyRule {
    pub from: Option<String>,
    pub to: Option<String>,
    pub table: u32,
}

pub async fn routing_policy_rule_add(rule: &RoutingPolicyRule) -> Result<()> {
    let links = super::acquire_links().await?;
    if links.links_by_mac.len() < 2 {
        return Ok(());
    }

    if rule_exists(rule).await? {
        return Ok(());
    }

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut request = handle
        .rule()
        .add()
        .v4()
        .table_id(rule.table)
        .action(RuleAction::ToTable);

    if let Some(ref from) = rule.from {
        let ip: Ipv4Addr = from.parse()?;
        request = request.source_prefix(ip, 32);
    }

    if let Some(ref to) = rule.to {
        let ip: Ipv4Addr = to.parse()?;
        request = request.destination_prefix(ip, 32);
    }

    let result = request.execute().await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("File exists") || err_str.contains("EEXIST") {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add routing policy rule: {}", e))
            }
        }
    }
}

pub async fn routing_policy_rule_remove(rule: &RoutingPolicyRule) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut message = RuleMessage::default();
    message.header.family = rtnetlink::packet_route::AddressFamily::Inet;
    message.header.action = RuleAction::ToTable;

    if rule.table > 255 {
        message.attributes.push(RuleAttribute::Table(rule.table));
    } else {
        message.header.table = rule.table as u8;
    }

    if let Some(ref from) = rule.from {
        let ip: Ipv4Addr = from.parse()?;
        message.header.src_len = 32;
        message.attributes.push(RuleAttribute::Source(ip.into()));
    }

    if let Some(ref to) = rule.to {
        let ip: Ipv4Addr = to.parse()?;
        message.header.dst_len = 32;
        message
            .attributes
            .push(RuleAttribute::Destination(ip.into()));
    }

    handle.rule().del(message).execute().await?;

    Ok(())
}

async fn rule_exists(rule: &RoutingPolicyRule) -> Result<bool> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut rule_stream = handle.rule().get(rtnetlink::IpVersion::V4).execute();

    while let Some(rule_msg) = rule_stream.try_next().await? {
        // Tables above 255 are carried in the FRA_TABLE attribute; the
        // 8-bit header field is then RT_TABLE_COMPAT (252).
        let table = rule_msg
            .attributes
            .iter()
            .find_map(|attr| match attr {
                RuleAttribute::Table(t) => Some(*t),
                _ => None,
            })
            .unwrap_or(rule_msg.header.table as u32);
        if table != rule.table {
            continue;
        }

        let mut src_ip: Option<String> = None;
        let mut dst_ip: Option<String> = None;

        for attr in &rule_msg.attributes {
            match attr {
                RuleAttribute::Source(ip) => src_ip = Some(ip.to_string()),
                RuleAttribute::Destination(ip) => dst_ip = Some(ip.to_string()),
                _ => {}
            }
        }

        let from_matches = match (&rule.from, &src_ip) {
            (Some(from), Some(src)) => from == src,
            (None, None) => true,
            _ => false,
        };

        let to_matches = match (&rule.to, &dst_ip) {
            (Some(to), Some(dst)) => to == dst,
            (None, None) => true,
            _ => false,
        };

        if from_matches && to_matches {
            return Ok(true);
        }
    }

    Ok(false)
}
