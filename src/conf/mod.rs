// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;
use tracing::Level;

pub const VERSION: &str = "0.3.0";
pub const CONF_FILE: &str = "cloud-network";
pub const CONF_PATH: &str = "/etc/cloud-network";
pub const SYSTEM_STATE_DIR: &str = "/run/cloud-network";
pub const LINK_STATE_DIR: &str = "/run/cloud-network/links";
pub const ROUTE_TABLE_BASE: u32 = 9999;

pub const DEFAULT_HTTP_REQUEST_TIMEOUT: u64 = 10000;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub logging: LoggingConfig,
    pub server: ServerConfig,
    pub metadata: MetadataConfig,
    pub network: NetworkConfig,
    pub cloud: CloudConfig,
    pub security: SecurityConfig,
    pub state: StateConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<String>,
    pub timestamps: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub listen: ListenConfig,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ListenConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_file: String,
    pub key_file: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MetadataConfig {
    pub refresh_interval: String,
    pub request_timeout: String,
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub backoff: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub interfaces: InterfacesConfig,
    pub primary: PrimaryConfig,
    pub routing: RoutingConfig,
    pub mtu: MtuConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct InterfacesConfig {
    pub enabled: Vec<String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PrimaryConfig {
    pub enabled: bool,
    pub interface: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RoutingConfig {
    pub table_base: u32,
    pub policy_routing: bool,
    pub manage_default_routes: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MtuConfig {
    pub auto_configure: bool,
    pub override_value: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CloudConfig {
    pub auto_detect: bool,
    pub provider: Option<String>,
    pub azure: AzureCloudConfig,
    pub aws: AwsCloudConfig,
    pub gcp: GcpCloudConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AzureCloudConfig {
    pub api_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AwsCloudConfig {
    pub imds_version: u8,
    pub token_ttl: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GcpCloudConfig {
    pub recursive: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub user: String,
    pub capabilities: Vec<String>,
    pub watchdog: WatchdogConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WatchdogConfig {
    pub enabled: bool,
    pub interval: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StateConfig {
    pub directory: String,
    pub persist_metadata: bool,
    pub per_interface_files: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FeaturesConfig {
    pub network_events: bool,
    pub cleanup_stale: bool,
    pub ipv6: bool,
    pub health_check: bool,
}

// Default implementations
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "text".to_string(),
            file: None,
            timestamps: false,
        }
    }
}

impl Default for ListenConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 5209,
        }
    }
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            refresh_interval: "300s".to_string(),
            request_timeout: "10s".to_string(),
            retry: RetryConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            backoff: "5s".to_string(),
        }
    }
}

impl Default for PrimaryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interface: None,
        }
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            table_base: 9999,
            policy_routing: true,
            manage_default_routes: true,
        }
    }
}

impl Default for MtuConfig {
    fn default() -> Self {
        Self {
            auto_configure: true,
            override_value: None,
        }
    }
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            provider: None,
            azure: AzureCloudConfig::default(),
            aws: AwsCloudConfig::default(),
            gcp: GcpCloudConfig::default(),
        }
    }
}

impl Default for AzureCloudConfig {
    fn default() -> Self {
        Self {
            api_version: "2021-02-01".to_string(),
        }
    }
}

impl Default for AwsCloudConfig {
    fn default() -> Self {
        Self {
            imds_version: 1,
            token_ttl: None,
        }
    }
}

impl Default for GcpCloudConfig {
    fn default() -> Self {
        Self { recursive: true }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            user: "cloud-network".to_string(),
            capabilities: vec!["CAP_NET_ADMIN".to_string()],
            watchdog: WatchdogConfig::default(),
        }
    }
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: "30s".to_string(),
        }
    }
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            directory: "/run/cloud-network".to_string(),
            persist_metadata: true,
            per_interface_files: true,
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            network_events: true,
            cleanup_stale: true,
            ipv6: false,
            health_check: true,
        }
    }
}

// Implementation methods
impl Config {
    pub fn parse() -> Result<Self> {
        let config_path = std::path::Path::new(CONF_PATH).join(format!("{}.yaml", CONF_FILE));

        let config_content = match std::fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(_) => {
                tracing::warn!("Failed to read config file, using defaults");
                return Ok(Self::default());
            }
        };

        let config: Config =
            serde_yaml::from_str(&config_content).context("Failed to parse config file")?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate refresh interval
        parse_duration(&self.metadata.refresh_interval)
            .context("Invalid metadata refresh_interval")?;

        // Validate request timeout
        parse_duration(&self.metadata.request_timeout)
            .context("Invalid metadata request_timeout")?;

        // Validate watchdog interval
        parse_duration(&self.security.watchdog.interval).context("Invalid watchdog interval")?;

        // Validate port
        if self.server.listen.port == 0 {
            return Err(anyhow::anyhow!("Invalid server port"));
        }

        Ok(())
    }

    pub fn get_log_level(&self) -> Level {
        match self.logging.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => {
                tracing::warn!(
                    "Unknown log level '{}', defaulting to info",
                    self.logging.level
                );
                Level::INFO
            }
        }
    }

    pub fn get_refresh_duration(&self) -> Duration {
        parse_duration(&self.metadata.refresh_interval).unwrap_or_else(|_| Duration::from_secs(300))
    }

    pub fn get_request_timeout(&self) -> Duration {
        parse_duration(&self.metadata.request_timeout).unwrap_or_else(|_| Duration::from_secs(10))
    }

    pub fn get_watchdog_interval(&self) -> Duration {
        parse_duration(&self.security.watchdog.interval).unwrap_or_else(|_| Duration::from_secs(30))
    }

    pub fn get_listen_addr(&self) -> String {
        format!("{}:{}", self.server.listen.address, self.server.listen.port)
    }

    pub fn get_supplementary_interfaces(&self) -> String {
        self.network.interfaces.enabled.join(" ")
    }
}

fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow::anyhow!("empty duration"));
    }

    let unit_start = s.char_indices().last().map(|(i, _)| i).unwrap_or(0);
    let (num_str, unit) = s.split_at(unit_start);
    let num: u64 = num_str.parse().context("invalid duration number")?;

    match unit {
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 3600)),
        "d" => Ok(Duration::from_secs(num * 86400)),
        _ => Err(anyhow::anyhow!("invalid duration unit: {}", unit)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("300s").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
    }

    #[test]
    fn test_parse_duration_multibyte_does_not_panic() {
        assert!(parse_duration("5\u{00e9}").is_err());
        assert!(parse_duration("\u{00e9}").is_err());
    }

    #[test]
    fn test_shipped_config_parses() {
        let s = include_str!("../../distribution/etc/cloud-network/config.yaml");
        let config: Config = serde_yaml::from_str(s).expect("shipped config.yaml must parse");
        config.validate().unwrap();
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.server.listen.port, 5209);
        assert_eq!(config.network.routing.table_base, 9999);
    }
}
