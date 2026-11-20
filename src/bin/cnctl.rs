// SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0

use clap::{Parser, Subcommand};
use cloud_netconfig::*;

#[derive(Parser)]
#[command(name = "cnctl")]
#[command(version = conf::VERSION)]
#[command(about = "Cloud Network Configuration Control", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show status information
    Status {
        /// What to show: system, network, or all
        #[arg(value_parser = ["system", "network", "all"], default_value = "all")]
        target: String,
    },

    /// Apply configuration from file
    Apply {
        /// Configuration file path
        #[arg(short, long, default_value = "/etc/cloud-network/cloud-network.yaml")]
        config: String,

        /// Dry-run mode (validate only)
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Reload daemon configuration
    Reload {
        /// Force reload even if config hasn't changed
        #[arg(short, long)]
        force: bool,
    },

    /// Show daemon version
    Version,
}

async fn fetch_metadata(endpoint: &str) -> anyhow::Result<serde_json::Value> {
    let config = conf::Config::parse().unwrap_or_default();
    let listen_addr = config.get_listen_addr();
    let url = format!("http://{}{}", listen_addr, endpoint);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let data = response.json::<serde_json::Value>().await?;

    Ok(data)
}

async fn show_system_status() -> anyhow::Result<()> {
    let kind = cloud::detect_cloud();

    println!("Cloud Provider: {}", kind.as_str());
    println!();

    match kind {
        cloud::CloudProvider::Azure => {
            if let Ok(data) = fetch_metadata("/api/cloud/system").await {
                if let Some(obj) = data.as_object() {
                    if let Some(name) = obj.get("name") {
                        println!("          Name: {}", name.as_str().unwrap_or(""));
                    }
                    if let Some(location) = obj.get("location") {
                        println!("      Location: {}", location.as_str().unwrap_or(""));
                    }
                    if let Some(vm_id) = obj.get("vmId") {
                        println!("         VM Id: {}", vm_id.as_str().unwrap_or(""));
                    }
                    if let Some(vm_size) = obj.get("vmSize") {
                        println!("       VM Size: {}", vm_size.as_str().unwrap_or(""));
                    }
                    if let Some(subscription_id) = obj.get("subscriptionId") {
                        println!(
                            "Subscription Id: {}",
                            subscription_id.as_str().unwrap_or("")
                        );
                    }
                }
            }
        }
        cloud::CloudProvider::AWS => {
            if let Ok(data) = fetch_metadata("/api/cloud/system").await {
                if let Some(obj) = data.as_object() {
                    if let Some(instance_id) = obj.get("instance_id") {
                        println!("   Instance Id: {}", instance_id.as_str().unwrap_or(""));
                    }
                    if let Some(instance_type) = obj.get("instance_type") {
                        println!(" Instance Type: {}", instance_type.as_str().unwrap_or(""));
                    }
                    if let Some(local_ipv4) = obj.get("local_ipv4") {
                        println!("    Local IPv4: {}", local_ipv4.as_str().unwrap_or(""));
                    }
                    if let Some(public_ipv4) = obj.get("public_ipv4") {
                        println!("   Public IPv4: {}", public_ipv4.as_str().unwrap_or(""));
                    }
                }
            }
        }
        cloud::CloudProvider::GCP => {
            if let Ok(data) = fetch_metadata("/api/cloud/system").await {
                if let Some(obj) = data.as_object() {
                    if let Some(id) = obj.get("id") {
                        println!("            Id: {}", id.as_str().unwrap_or(""));
                    }
                    if let Some(hostname) = obj.get("hostname") {
                        println!("      Hostname: {}", hostname.as_str().unwrap_or(""));
                    }
                    if let Some(machine_type) = obj.get("machineType") {
                        println!("  Machine Type: {}", machine_type.as_str().unwrap_or(""));
                    }
                }
            }
        }
        _ => {
            println!("No detailed information available");
        }
    }

    Ok(())
}

async fn show_network_status() -> anyhow::Result<()> {
    let links = network::acquire_links().await?;

    println!("Network Interfaces:");
    println!();

    for link in links.links_by_mac.values() {
        println!("       Name: {}", link.name);
        println!("MAC Address: {}", link.mac);
        println!("      State: {}", link.oper_state);
        println!("        MTU: {}", link.mtu);

        if let Ok(addresses) = network::get_ipv4_addresses(&link.name).await {
            for (addr, _) in addresses {
                println!(" Private IP: {}", addr);
            }
        }

        println!();
    }

    Ok(())
}

async fn show_daemon_status() -> anyhow::Result<()> {
    if let Ok(data) = fetch_metadata("/api/status").await {
        if let Some(obj) = data.as_object() {
            if let Some(status) = obj.get("status") {
                println!("Daemon Status: {}", status.as_str().unwrap_or("unknown"));
            }
            if let Some(provider) = obj.get("provider") {
                println!("     Provider: {}", provider.as_str().unwrap_or("unknown"));
            }
            if let Some(version) = obj.get("version") {
                println!("      Version: {}", version.as_str().unwrap_or("unknown"));
            }
        }
    } else {
        println!("Daemon Status: not running");
    }

    Ok(())
}

async fn apply_config(config_path: &str, dry_run: bool) -> anyhow::Result<()> {
    println!("Reading configuration from: {}", config_path);

    // Read and parse the config file
    let config_content = std::fs::read_to_string(config_path)?;
    let config: conf::Config = serde_yaml::from_str(&config_content)?;

    if dry_run {
        println!("✓ Configuration validation passed");
        println!("\nConfiguration summary:");
        println!("  Log level: {}", config.logging.level);
        println!("  Listen: {}", config.get_listen_addr());
        println!("  Refresh interval: {:?}", config.get_refresh_duration());
        println!("  Route table base: {}", config.network.routing.table_base);

        if !config.network.interfaces.enabled.is_empty() {
            println!(
                "  Supplementary interfaces: {}",
                config.network.interfaces.enabled.join(", ")
            );
        }
    } else {
        println!("✓ Configuration validated");
        println!("\nTo apply this configuration:");
        println!("  1. Copy to /etc/cloud-network/cloud-network.yaml");
        println!("  2. Run: sudo systemctl restart cloud-netconfigd");
    }

    Ok(())
}

async fn reload_daemon(force: bool) -> anyhow::Result<()> {
    println!("Reloading daemon configuration...");

    if force {
        println!("Force reload requested");
    }

    // Send reload signal to daemon (would need IPC implementation)
    println!("\nCurrently, please reload using:");
    println!("  sudo systemctl reload cloud-netconfigd");

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Status { target } => match target.as_str() {
            "system" => show_system_status().await?,
            "network" => show_network_status().await?,
            "all" => {
                show_daemon_status().await?;
                println!();
                show_system_status().await?;
                println!();
                show_network_status().await?;
            }
            _ => {
                eprintln!("Unknown target: {}", target);
                std::process::exit(1);
            }
        },

        Commands::Apply { config, dry_run } => {
            apply_config(config, *dry_run).await?;
        }

        Commands::Reload { force } => {
            reload_daemon(*force).await?;
        }

        Commands::Version => {
            println!("cnctl version {}", conf::VERSION);
            println!("License:        Zyvor Production License v1.0");
        }
    }

    Ok(())
}
