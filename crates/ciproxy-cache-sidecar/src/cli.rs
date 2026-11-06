//! Machine and VM proxy CLI configuration and listener startup.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use ciproxy_cache_sidecar::clock::{Clock, SystemClock};
use ciproxy_cache_sidecar::machine::MachineCache;
use ciproxy_cache_sidecar::runtime_token::TokenVerifier;
use ciproxy_cache_sidecar::runtime_token::keys::{CachedKeys, SigningKeys};
use ciproxy_cache_sidecar::runtime_token::source::{HttpKeys, VerificationConfig};
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
    /// Reclaim offline archive and staging files while preserving report history.
    Reclaim {
        #[arg(long)]
        cache_dir: PathBuf,
    },
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
        /// Machine-owned key source file for isolated operator smoke tests.
        #[arg(long)]
        verification_config: Option<PathBuf>,
    },
    /// Forward this VM's cache requests and return misses during machine outages.
    Proxy {
        #[arg(long, default_value = "127.0.0.1:8230")]
        listen: SocketAddr,
        #[arg(long, required_unless_present = "config_stdin")]
        config: Option<PathBuf>,
        /// Receive credentials from a parent pipe, without writing a file.
        #[arg(long, conflicts_with = "config")]
        config_stdin: bool,
        /// Acknowledge the bound listener to the supervising guest.
        #[arg(long)]
        ready_stdout: bool,
    },
}

pub(crate) async fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Reclaim { cache_dir } => {
            let store = DiskStorage::new(cache_dir, 0);
            match store.clear().await {
                Ok(()) => Ok(()),
                Err(error) => Err(CliError::Cache(error)),
            }
        }
        Command::Machine {
            listen,
            admin_listen,
            cache_dir,
            capacity_bytes,
            admin_token_file,
            verification_config,
        } => {
            let network = network()?;
            let token = match fs::read_to_string(admin_token_file).await {
                Ok(value) => value,
                Err(error) => return Err(CliError::Io(error)),
            };
            let store = Arc::new(DiskStorage::new(cache_dir, capacity_bytes));
            if store.recover().await.is_err() {
                tracing::warn!("Local cache disk unavailable; GitHub fallback remains available");
            }
            let verification_config = verification_config.or_else(|| {
                std::env::var_os("CIPROXY_CACHE_VERIFICATION_CONFIG").map(PathBuf::from)
            });
            let source = match verification_config {
                Some(path) => {
                    let bytes = match fs::read(path).await {
                        Ok(bytes) => bytes,
                        Err(error) => return Err(CliError::Io(error)),
                    };
                    let config: VerificationConfig = match serde_json::from_slice(&bytes) {
                        Ok(config) => config,
                        Err(error) => return Err(CliError::Json(error)),
                    };
                    HttpKeys::configured(network.clone(), config)
                }
                None => HttpKeys::github(network.clone()),
            };
            let clock: Arc<dyn Clock> = Arc::new(SystemClock);
            let keys: Arc<dyn SigningKeys> =
                Arc::new(CachedKeys::new(Arc::new(source), clock.clone()));
            let refresh_keys = keys.clone();
            let refresh = tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    refresh_keys.refresh().await;
                }
            });
            let service: Arc<dyn Service> = Arc::new(MachineCache::new(
                store,
                network,
                clock.clone(),
                token.trim().to_owned(),
                Arc::new(TokenVerifier::new(keys, clock)),
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
            let result = match tokio::try_join!(jobs, admin) {
                Ok(_) => Ok(()),
                Err(error) => Err(CliError::Io(error)),
            };
            refresh.abort();
            result
        }
        Command::Proxy {
            listen,
            config,
            config_stdin,
            ready_stdout,
        } => crate::proxy_cli::run(listen, config, config_stdin, ready_stdout, network()?).await,
    }
}

fn network() -> Result<Arc<dyn Transport>> {
    match HttpTransport::new() {
        Ok(network) => Ok(Arc::new(network)),
        Err(error) => Err(CliError::Cache(error)),
    }
}
