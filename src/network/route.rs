// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0

use anyhow::{anyhow, Result};
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use rtnetlink::packet_route::route::{RouteAddress, RouteAttribute, RouteMessage};
use rtnetlink::packet_route::AddressFamily;
use rtnetlink::RouteMessageBuilder;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct Route {
    pub table: u32,
    pub if_index: u32,
    pub gw: String,
}

fn ipv4_from_route_address(addr: &RouteAddress) -> Option<String> {
    match addr {
        RouteAddress::Inet(addr) => Some(addr.to_string()),
        _ => None,
    }
}

async fn iter_ipv4_routes(
) -> Result<impl Iterator<Item = rtnetlink::packet_route::route::RouteMessage>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut get_msg = RouteMessage::default();
    get_msg.header.address_family = AddressFamily::Inet;
    let mut route_stream = handle.route().get(get_msg).execute();

    let mut routes = Vec::new();
    while let Some(route_msg) = route_stream.try_next().await? {
        routes.push(route_msg);
    }
    Ok(routes.into_iter())
}

pub async fn get_default_ipv4_gateway() -> Result<String> {
    for route_msg in iter_ipv4_routes().await? {
        if route_msg.header.destination_prefix_length != 0 {
            continue;
        }

        for attr in &route_msg.attributes {
            if let RouteAttribute::Gateway(gw) = attr {
                if let Some(ip) = ipv4_from_route_address(gw) {
                    return Ok(ip);
                }
            }
        }
    }

    Err(anyhow!("Default gateway not found"))
}

pub async fn get_default_ipv4_gateway_by_link(if_index: u32) -> Result<String> {
    for route_msg in iter_ipv4_routes().await? {
        if route_msg.header.destination_prefix_length != 0 {
            continue;
        }

        let matches_link = route_msg
            .attributes
            .iter()
            .any(|attr| matches!(attr, RouteAttribute::Oif(idx) if *idx == if_index));

        if !matches_link {
            continue;
        }

        for attr in &route_msg.attributes {
            if let RouteAttribute::Gateway(gw) = attr {
                if let Some(ip) = ipv4_from_route_address(gw) {
                    return Ok(ip);
                }
            }
        }
    }

    Err(anyhow!("Default gateway not found for link {}", if_index))
}

pub async fn get_ipv4_gateway_by_link(if_index: u32) -> Result<String> {
    for route_msg in iter_ipv4_routes().await? {
        let matches_link = route_msg
            .attributes
            .iter()
            .any(|attr| matches!(attr, RouteAttribute::Oif(idx) if *idx == if_index));

        if !matches_link {
            continue;
        }

        for attr in &route_msg.attributes {
            if let RouteAttribute::Gateway(gw) = attr {
                if let Some(ip) = ipv4_from_route_address(gw) {
                    return Ok(ip);
                }
            }
        }
    }

    Err(anyhow!("Gateway not found for link {}", if_index))
}

pub async fn route_add(route: &Route) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let gw: Ipv4Addr = route.gw.parse()?;

    let route_msg = RouteMessageBuilder::<Ipv4Addr>::new()
        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0)
        .gateway(gw)
        .output_interface(route.if_index)
        .table_id(route.table)
        .build();

    let result = handle.route().add(route_msg).execute().await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("File exists") || err_str.contains("EEXIST") {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add route: {}", e))
            }
        }
    }
}

pub async fn route_remove(route: &Route) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let gw: Ipv4Addr = route.gw.parse()?;

    let route_msg = RouteMessageBuilder::<Ipv4Addr>::new()
        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0)
        .gateway(gw)
        .output_interface(route.if_index)
        .table_id(route.table)
        .build();

    handle.route().del(route_msg).execute().await?;

    Ok(())
}
