use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::signal;
use tracing::{info, warn};

use sovrn_dht::{DhtService, DhtConfig};

#[derive(Debug)]
struct Args {
    config: PathBuf,
}

fn parse_args() -> Args {
    let mut config = PathBuf::from("/etc/sovrn/dht.toml");
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
    Args { config }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = parse_args();
    info!("sovrn-dht starting with config {:?}", args.config);

    let config_str = std::fs::read_to_string(&args.config).unwrap_or_default();
    let config: DhtConfig = toml::from_str(&config_str).unwrap_or_default();

    let socket_dir = std::path::Path::new(&config.sockets_dir);
    std::fs::create_dir_all(socket_dir)?;

    let service = DhtService::new(config)?;
    let service = Arc::new(service);

    // Initialize database
    {
        let svc = service.clone();
        svc.init_db()?;
    }

    // Start Yggdrasil listener
    let ygg_handle = {
        let svc = service.clone();
        tokio::spawn(async move {
            if let Err(e) = sovrn_dht::ygg::listen_yggdrasil(svc).await {
                warn!("Yggdrasil listener error: {}", e);
            }
        })
    };

    // Start JSON-RPC server on Unix socket
    let rpc_handle = {
        let svc = service.clone();
        tokio::spawn(async move {
            if let Err(e) = sovrn_dht::rpc::serve(svc).await {
                warn!("RPC server error: {}", e);
            }
        })
    };

    // Notify systemd we're ready
    sd_notify::notify(true, &[sd_notify::NotifyState::Ready])?;

    info!("sovrn-dht is running");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutting down sovrn-dht");

    ygg_handle.abort();
    rpc_handle.abort();

    Ok(())
}
