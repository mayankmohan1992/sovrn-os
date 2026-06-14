//! Sovrn Presence Service entry point

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use tokio::signal;
use tracing::info;

use sovrn_presence::{PresenceService, PresenceConfig};

fn parse_args() -> PathBuf {
    let mut config = PathBuf::from("/etc/sovrn/presence.toml");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" | "-c" => { if let Some(p) = args.next() { config = PathBuf::from(p); } }
            _ => {}
        }
    }
    config
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")))
        .init();

    let config_path = parse_args();
    info!("sovrn-presence starting with config {:?}", config_path);
    let config_str = std::fs::read_to_string(&config_path)?;
    let config: PresenceConfig = toml::from_str(&config_str).unwrap_or_default();

    std::fs::create_dir_all(&config.sockets_dir)?;
    std::fs::create_dir_all(&config.data_dir)?;

    let service = PresenceService::new(config)?;
    service.init_db()?;
    let service = Arc::new(service);

    let rpc_handle = {
        let svc = service.clone();
        tokio::spawn(async move {
            if let Err(e) = sovrn_presence::rpc::serve(svc).await {
                tracing::error!("Presence RPC error: {}", e);
            }
        })
    };

    sd_notify::notify(true, &[sd_notify::NotifyState::Ready])?;
    info!("sovrn-presence is running");

    signal::ctrl_c().await?;
    info!("Shutting down sovrn-presence");
    rpc_handle.abort();
    Ok(())
}
