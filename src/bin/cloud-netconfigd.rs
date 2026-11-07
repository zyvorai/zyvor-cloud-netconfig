// SPDX-License-Identifier: Apache-2.0

use axum::{routing::get, Router};
use cloud_netconfig::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

async fn cloud_network_begin(env: Arc<Mutex<provider::Environment>>) -> anyhow::Result<()> {
    let mut env_guard = env.lock().await;

    tracing::debug!("Connecting to metadata server ({}) ...", env_guard.kind);

    provider::acquire_cloud_metadata(&mut env_guard).await?;

    tracing::debug!("Configuring network from ({}) metadata", env_guard.kind);

    provider::configure_network_metadata(&mut env_guard).await?;

    tracing::debug!("Saving ({}) metadata", env_guard.kind);

    provider::save_metadata(&env_guard).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn status_endpoint(env: Arc<Mutex<provider::Environment>>) -> axum::Json<serde_json::Value> {
    let env_guard = env.lock().await;
    axum::Json(serde_json::json!({
        "status": "running",
        "provider": env_guard.kind.as_str(),
        "version": conf::VERSION
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse configuration
    let config = conf::Config::parse()?;

    // Setup logging
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    match config.logging.format.as_str() {
        "json" => {
            let layer = if config.logging.timestamps {
                fmt::layer().json().boxed()
            } else {
                fmt::layer().json().without_time().boxed()
            };
            tracing_subscriber::registry()
                .with(layer)
                .with(filter)
                .init();
        }
        _ => {
            let layer = if config.logging.timestamps {
                fmt::layer().boxed()
            } else {
                fmt::layer().without_time().boxed()
            };
            tracing_subscriber::registry()
                .with(layer)
                .with(filter)
                .init();
        }
    }

    tracing::info!("cloud-netconfig v{}", conf::VERSION);
    tracing::debug!("Configuration loaded: {:?}", config);

    // Detect cloud environment
    let kind = if config.cloud.auto_detect {
        cloud::detect_cloud()
    } else if let Some(ref provider_name) = config.cloud.provider {
        match provider_name.as_str() {
            "azure" => cloud::CloudProvider::Azure,
            "aws" => cloud::CloudProvider::AWS,
            "gcp" => cloud::CloudProvider::GCP,
            "alibaba" => cloud::CloudProvider::Alibaba,
            "oracle" => cloud::CloudProvider::Oracle,
            "digitalocean" => cloud::CloudProvider::DigitalOcean,
            _ => {
                tracing::error!("Unknown cloud provider: {}", provider_name);
                cloud::CloudProvider::None
            }
        }
    } else {
        cloud::CloudProvider::None
    };

    if kind == cloud::CloudProvider::None {
        tracing::error!("Failed to detect cloud environment, Aborting ...");
        std::process::exit(1);
    }

    tracing::info!("Detected cloud environment: {}", kind);

    // Initialize provider environment. Detection recognises more clouds than
    // there are providers for, so fail cleanly instead of panicking.
    let Some(env) = provider::Environment::new(kind, &config) else {
        tracing::error!(
            "Cloud environment '{}' is detected but not supported yet (supported: azure, aws, gcp), Aborting ...",
            kind
        );
        std::process::exit(1);
    };

    // Handle security and privilege dropping
    if let Ok(cred) = system::get_user_credentials(None) {
        if cred.uid.is_root() {
            tracing::info!(
                "Running as root, attempting to drop privileges to user: {}",
                config.security.user
            );

            if let Ok(cloud_cred) = system::get_user_credentials(Some(&config.security.user)) {
                let _ = system::create_state_dirs(
                    kind.as_str(),
                    cloud_cred.uid.as_raw(),
                    cloud_cred.gid.as_raw(),
                );

                let _ = system::enable_keep_capability();
                let _ = system::switch_user(&cloud_cred);
                let _ = system::disable_keep_capability();
                let _ = system::apply_capability(&cloud_cred);

                tracing::info!(
                    "Successfully dropped privileges to user: {}",
                    config.security.user
                );
            } else {
                tracing::warn!(
                    "User '{}' not found, continuing as root",
                    config.security.user
                );
            }
        }
    }

    // Wrap environment in Arc<Mutex> for sharing
    let env = Arc::new(Mutex::new(env));

    // Start network event watching if enabled
    if config.features.network_events {
        tracing::info!("Network event watching enabled");
        provider::watch_network(env.clone()).await;
    }

    // Initial configuration
    if let Err(e) = cloud_network_begin(env.clone()).await {
        tracing::error!("Error during initial configuration: {}", e);
    } else {
        // Configure supplementary interfaces
        let supplementary = config.get_supplementary_interfaces();
        if !supplementary.is_empty() {
            tracing::info!("Configuring supplementary interfaces: {}", supplementary);
            network::configure_supplementary_links(&supplementary)
                .await
                .ok();
        }
    }

    // Start periodic refresh timer
    let refresh_duration = config.get_refresh_duration();
    let env_clone = env.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(refresh_duration);
        loop {
            interval.tick().await;
            tracing::debug!("Periodic metadata refresh triggered");
            if let Err(e) = cloud_network_begin(env_clone.clone()).await {
                tracing::error!("Error during periodic refresh: {}", e);
            }
        }
    });

    // Setup HTTP server
    let env_for_status = env.clone();
    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/api/status",
            get(move || status_endpoint(env_for_status.clone())),
        )
        .route("/api/cloud/status", get(health_check));

    let listen_addr = config.get_listen_addr();
    let addr: SocketAddr = listen_addr.parse()?;

    tracing::info!("HTTP API server listening on {}", listen_addr);

    // Notify systemd that we're ready
    if let Err(e) = libsystemd::daemon::notify(false, &[libsystemd::daemon::NotifyState::Ready]) {
        tracing::warn!("Failed to notify systemd: {}", e);
    }

    // Start watchdog thread if enabled
    if config.security.watchdog.enabled {
        let watchdog_interval = config.get_watchdog_interval();
        tracing::info!(
            "Systemd watchdog enabled with interval: {:?}",
            watchdog_interval
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(watchdog_interval);
            loop {
                interval.tick().await;
                let _ =
                    libsystemd::daemon::notify(false, &[libsystemd::daemon::NotifyState::Watchdog]);
            }
        });
    }

    // Setup signal handlers for graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("Received shutdown signal, stopping...");
    };

    // Start HTTP server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Notify systemd that we're stopping
    let _ = libsystemd::daemon::notify(false, &[libsystemd::daemon::NotifyState::Stopping]);

    tracing::info!("Shutdown complete");
    Ok(())
}
