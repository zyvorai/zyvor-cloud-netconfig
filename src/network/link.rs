// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Context, Result};
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use rtnetlink::packet_route::link::LinkAttribute;
use rtnetlink::LinkUnspec;

#[derive(Debug, Clone)]
pub struct Link {
    pub name: String,
    pub ifindex: u32,
    pub oper_state: String,
    pub mac: String,
    pub mtu: u32,
    pub addresses: Option<std::collections::HashMap<String, bool>>,
}

#[derive(Debug, Clone, Default)]
pub struct Links {
    pub links_by_mac: std::collections::HashMap<String, Link>,
}

impl Links {
    pub fn new() -> Self {
        Self {
            links_by_mac: std::collections::HashMap::new(),
        }
    }

    /// Records `link` unless a link with the same MAC was already seen.
    ///
    /// Several links can share a MAC: on Azure with Accelerated Networking the
    /// SR-IOV VF (e.g. `enP1s1`) carries the synthetic `eth0`'s MAC and is
    /// dumped after it, and VLAN sub-interfaces inherit their parent's MAC.
    /// Keep the first (lowest ifindex) link, as the original Go implementation
    /// did, so addresses and routes go on the primary interface.
    pub fn insert_first(&mut self, link: Link) {
        self.links_by_mac.entry(link.mac.clone()).or_insert(link);
    }
}

pub async fn acquire_links() -> Result<Links> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = Links::new();
    let mut link_stream = handle.link().get().execute();

    while let Some(link_msg) = link_stream.try_next().await? {
        let mut name = String::new();
        let mut mac = String::new();
        let mut mtu = 0u32;
        let mut oper_state = "unknown".to_string();

        for attr in &link_msg.attributes {
            match attr {
                LinkAttribute::IfName(n) => name = n.clone(),
                LinkAttribute::Address(addr) if addr.len() == 6 => {
                    mac = format!(
                        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                        addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]
                    );
                }
                LinkAttribute::Mtu(m) => mtu = *m,
                LinkAttribute::OperState(state) => oper_state = format!("{:?}", state),
                _ => {}
            }
        }

        if name == "lo" || mac.is_empty() {
            continue;
        }

        let link = Link {
            name,
            ifindex: link_msg.header.index,
            oper_state,
            mac,
            mtu,
            addresses: None,
        };

        links.insert_first(link);
    }

    Ok(links)
}

pub async fn get_link_mac_by_index(links: &Links, if_index: u32) -> Result<String> {
    links
        .links_by_mac
        .values()
        .find(|link| link.ifindex == if_index)
        .map(|link| link.mac.clone())
        .ok_or_else(|| anyhow!("not found"))
}

pub async fn get_link_name_by_index(if_index: u32) -> Result<String> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut link_stream = handle.link().get().match_index(if_index).execute();

    if let Some(link_msg) = link_stream.try_next().await? {
        for attr in &link_msg.attributes {
            if let LinkAttribute::IfName(n) = attr {
                return Ok(n.clone());
            }
        }
        return Err(anyhow!("Link name not found"));
    }

    Err(anyhow!("Link with index {} not found", if_index))
}

pub async fn get_link_index_by_name(name: &str) -> Result<u32> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut link_stream = handle.link().get().match_name(name.to_string()).execute();

    if let Some(link_msg) = link_stream.try_next().await? {
        return Ok(link_msg.header.index);
    }

    Err(anyhow!("Link '{}' not found", name))
}

pub async fn link_set_oper_state_up(if_index: u32) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    handle
        .link()
        .change(LinkUnspec::new_with_index(if_index).up().build())
        .execute()
        .await
        .context("Failed to bring link up")?;

    Ok(())
}

pub async fn link_set_mtu(if_index: u32, mtu: u32) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    handle
        .link()
        .change(LinkUnspec::new_with_index(if_index).mtu(mtu).build())
        .execute()
        .await
        .context("Failed to set MTU")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(name: &str, ifindex: u32, mac: &str) -> Link {
        Link {
            name: name.to_string(),
            ifindex,
            oper_state: "Up".to_string(),
            mac: mac.to_string(),
            mtu: 1500,
            addresses: None,
        }
    }

    #[test]
    fn keeps_first_link_for_shared_mac() {
        // Azure Accelerated Networking: the VF shares eth0's MAC and has a higher ifindex.
        let mut links = Links::new();
        links.insert_first(link("eth0", 2, "00:0d:3a:f8:06:ec"));
        links.insert_first(link("enP1s1", 3, "00:0d:3a:f8:06:ec"));

        assert_eq!(links.links_by_mac.len(), 1);
        assert_eq!(links.links_by_mac["00:0d:3a:f8:06:ec"].name, "eth0");
    }
}
