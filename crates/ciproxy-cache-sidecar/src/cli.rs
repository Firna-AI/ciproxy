//! Machine and VM proxy CLI configuration and listener startup.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use ciproxy_cache_sidecar::clock::SystemClock;
use ciproxy_cache_sidecar::machine::MachineCache;
use ciproxy_cache_sidecar::proxy::{ProxyConfig, VmProxy};
use ciproxy_cache_sidecar::service::{Service, admin_router, job_router};
use ciproxy_cache_sidecar::storage::disk::DiskStorage;
use ciproxy_cache_sidecar::storage::interface::Storage;
use ciproxy_cache_sidecar::transport::{HttpTransport, Transport};
use clap::{Parser, Subcommand};
use tokio::fs;
use tokio::net::TcpListener;

type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    #[error("[cache/cli] IO failure: {0}")]
    Io(std::io::Error),
    #[error("[cache/cli] configuration failure: {0}")]
    Json(serde_json::Error),
    #[error("[cache/cli] cache initialization failure: {0}")]
    Cache(ciproxy_cache_sidecar::error::CacheError),
}

#[derive(Parser)]
#[command(version, about = "Transparent GitHub Actions cache for one machine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the machine cache and its separate agent-only admin API.
    Machine {
        #[arg(long, default_value = "127.0.0.1:8228")]
        listen: SocketAddr,
        #[arg(long, default_value = "127.0.0.1:8229")]
        admin_listen: SocketAddr,
        #[arg(long)]
        cache_dir: PathBuf,
        #[arg(long)]
        capacity_bytes: u64,
        #[arg(long)]
        admin_token_file: PathBuf,
    },
    /// Forward this VM's cache requests with direct GitHub fallback.
    Proxy {
        #[arg(long, default_value = "127.0.0.1:8230")]
        listen: SocketAddr,
        #[arg(long)]
        config: PathBuf,
    },
}

pub(crate) async fn run() -> Result<()> {
    let network: Arc<dyn Transport> = match HttpTransport::new() {
        Ok(network) => Arc::new(network),
        Err(error) => return Err(CliError::Cache(error)),
    };
    match Cli::parse().command {
        Command::Machine {
            listen,
            admin_listen,
            cache_dir,
            capacity_bytes,
            admin_token_file,
        } => {
            let token = match fs::read_to_string(admin_token_file).await {
                Ok(value) => value,
                Err(error) => return Err(CliError::Io(error)),
            };
            let store = Arc::new(DiskStorage::new(cache_dir, capacity_bytes));
            if store.recover().await.is_err() {
                tracing::warn!("Local cache disk unavailable; GitHub fallback remains available");
            }
            let service: Arc<dyn Service> = Arc::new(MachineCache::new(
                store,
                network,
                Arc::new(SystemClock),
                token.trim().to_owned(),
            ));
            let job_listener = match TcpListener::bind(listen).await {
                Ok(value) => value,
                Err(error) => return Err(CliError::Io(error)),
            };
            let admin_listener = match TcpListener::bind(admin_listen).await {
                Ok(value) => value,
                Err(error) => return Err(CliError::Io(error)),
            };
            tracing::info!(%listen, %admin_listen, "Machine cache listening");
            let jobs = axum::serve(job_listener, job_router(service.clone()));
            let admin = axum::serve(admin_listener, admin_router(service));
            match tokio::try_join!(jobs, admin) {
                Ok(_) => Ok(()),
                Err(error) => Err(CliError::Io(error)),
            }
        }
        Command::Proxy { listen, config } => {
            let bytes = match fs::read(config).await {
                Ok(value) => value,
                Err(error) => return Err(CliError::Io(error)),
            };
            let config: ProxyConfig = match serde_json::from_slice(&bytes) {
                Ok(value) => value,
                Err(error) => return Err(CliError::Json(error)),
            };
            let service: Arc<dyn Service> = Arc::new(VmProxy::new(config, network));
            let listener = match TcpListener::bind(listen).await {
                Ok(value) => value,
                Err(error) => return Err(CliError::Io(error)),
            };
            tracing::info!(%listen, "VM cache proxy listening");
            match axum::serve(listener, job_router(service)).await {
                Ok(()) => Ok(()),
                Err(error) => Err(CliError::Io(error)),
            }
        }
    }
}
