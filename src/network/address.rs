// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use rtnetlink::packet_route::address::AddressAttribute;
use rtnetlink::packet_route::address::AddressMessage;
use rtnetlink::packet_route::AddressFamily;
use std::collections::HashMap;
use std::net::IpAddr;

pub async fn address_add(if_index: u32, address: &str) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let parts: Vec<&str> = address.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid address format, expected CIDR notation"
        ));
    }

    let ip: IpAddr = parts[0].parse()?;
    let prefix_len: u8 = parts[1].parse()?;

    let result = handle
        .address()
        .add(if_index, ip, prefix_len)
        .execute()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("File exists") || err_str.contains("EEXIST") {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add address: {}", e))
            }
        }
    }
}

pub async fn address_set(name: &str, address: &str) -> Result<()> {
    let if_index = super::get_link_index_by_name(name).await?;

    let parts: Vec<&str> = address.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid address format, expected CIDR notation"
        ));
    }

    let ip: IpAddr = parts[0].parse()?;
    let prefix_len: u8 = parts[1].parse()?;

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    handle
        .address()
        .add(if_index, ip, prefix_len)
        .replace()
        .execute()
        .await
        .context("Failed to set address")?;

    Ok(())
}

pub async fn get_ipv4_addresses(if_name: &str) -> Result<HashMap<String, bool>> {
    let if_index = super::get_link_index_by_name(if_name).await?;

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut addresses = HashMap::new();
    let mut addr_stream = handle
        .address()
        .get()
        .set_link_index_filter(if_index)
        .execute();

    while let Some(addr_msg) = addr_stream.try_next().await? {
        if addr_msg.header.family != AddressFamily::Inet {
            continue;
        }

        for attr in &addr_msg.attributes {
            if let AddressAttribute::Address(IpAddr::V4(v4)) = attr {
                let ip = v4.to_string();
                if !ip.is_empty() {
                    let prefix = addr_msg.header.prefix_len;
                    addresses.insert(format!("{}/{}", ip, prefix), true);
                }
            }
        }
    }

    Ok(addresses)
}

pub async fn address_remove(name: &str, address: &str) -> Result<()> {
    let if_index = super::get_link_index_by_name(name).await?;

    let parts: Vec<&str> = address.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid address format, expected CIDR notation"
        ));
    }

    let ip: IpAddr = parts[0].parse()?;
    let prefix_len: u8 = parts[1].parse()?;

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut message = AddressMessage::default();
    message.header.index = if_index;
    message.header.prefix_len = prefix_len;
    match ip {
        IpAddr::V4(v4) => {
            message.header.family = AddressFamily::Inet;
            message
                .attributes
                .push(AddressAttribute::Address(IpAddr::V4(v4)));
        }
        IpAddr::V6(v6) => {
            message.header.family = AddressFamily::Inet6;
            message
                .attributes
                .push(AddressAttribute::Address(IpAddr::V6(v6)));
        }
    }

    handle
        .address()
        .del(message)
        .execute()
        .await
        .context("Failed to remove address")?;

    Ok(())
}
