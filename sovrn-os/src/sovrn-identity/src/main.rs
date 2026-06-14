//! Sovrn Identity Service entry point

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::signal;
use tracing::info;

use sovrn_identity::{IdentityService, IdentityConfig};

fn parse_args() -> PathBuf {
    let mut config = PathBuf::from("/etc/sovrn/identity.toml");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                if let Some(path) = args.next() {
                    config = PathBuf::from(path);
                }
            }
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
    info!("sovrn-identity starting with config {:?}", config_path);

    let config_str = std::fs::read_to_string(&config_path)?;
    let config: IdentityConfig = toml::from_str(&config_str).unwrap_or_default();

    std::fs::create_dir_all(&config.sockets_dir)?;
    std::fs::create_dir_all(&config.data_dir)?;

    let service = IdentityService::new(config)?;
    service.init_db()?;
    let service = Arc::new(service);

    let rpc_handle = {
        let svc = service.clone();
        tokio::spawn(async move {
            if let Err(e) = sovrn_identity::rpc::serve(svc).await {
                tracing::error!("Identity RPC error: {}", e);
            }
        })
    };

    sd_notify::notify(true, &[sd_notify::NotifyState::Ready])?;
    info!("sovrn-identity is running");

    signal::ctrl_c().await?;
    info!("Shutting down sovrn-identity");

    rpc_handle.abort();
    Ok(())
}
