// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::net::{IpAddr, SocketAddr};

pub fn parse_ip(ip: &str) -> Result<IpAddr> {
    ip.parse::<IpAddr>().map_err(|_| anyhow!("invalid IP"))
}

pub fn parse_port(port: &str) -> Result<u16> {
    port.parse::<u16>().map_err(|_| anyhow!("invalid port"))
}

pub fn parse_ip_port(s: &str) -> Result<(String, String)> {
    let addr = s
        .parse::<SocketAddr>()
        .map_err(|_| anyhow!("invalid socket address"))?;

    Ok((addr.ip().to_string(), addr.port().to_string()))
}

/// Splits MAC address without ':' or '-' into MAC address format by inserting ':'
pub fn parse_mac(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % 2 == 0 {
            result.push(':');
        }
        result.push(c);
    }
    result
}

/// Normalizes a MAC address to the lowercase, colon-separated form used as the
/// key of `Links::links_by_mac` (e.g. `00:0d:3a:f8:06:ec`).
///
/// Accepts the bare upper-case form returned by the Azure Instance Metadata
/// Service (`000D3AF806EC`) as well as `-` or `:` separated forms.
pub fn normalize_mac(s: &str) -> String {
    let hex: String = s
        .chars()
        .filter(|c| *c != ':' && *c != '-')
        .collect::<String>()
        .to_ascii_lowercase();
    parse_mac(&hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mac() {
        assert_eq!(parse_mac("001122334455"), "00:11:22:33:44:55");
    }

    #[test]
    fn test_normalize_mac() {
        // Azure IMDS returns bare, upper-case MACs.
        assert_eq!(normalize_mac("000D3AF806EC"), "00:0d:3a:f8:06:ec");
        assert_eq!(normalize_mac("00-0D-3A-F8-06-EC"), "00:0d:3a:f8:06:ec");
        assert_eq!(normalize_mac("00:0d:3a:f8:06:ec"), "00:0d:3a:f8:06:ec");
    }

    #[test]
    fn test_parse_ip_port() {
        let (ip, port) = parse_ip_port("127.0.0.1:5209").unwrap();
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, "5209");
    }
}
