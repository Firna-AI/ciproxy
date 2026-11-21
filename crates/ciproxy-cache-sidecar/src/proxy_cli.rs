//! Private stdin configuration and acknowledged updates for the VM proxy.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use ciproxy_cache_sidecar::proxy::{ProxyControl, VmProxy};
use ciproxy_cache_sidecar::service::job_router;
use ciproxy_cache_sidecar::transport::Transport;
use ciproxy_protocol::runtime::{CacheProxyConfig, CacheProxyUpdate};
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};
use tokio::net::TcpListener;

use crate::cli::CliError;

pub(crate) async fn run(
    listen: SocketAddr,
    config: Option<PathBuf>,
    config_stdin: bool,
    ready_stdout: bool,
    network: Arc<dyn Transport>,
) -> Result<(), CliError> {
    let mut input = BufReader::new(tokio::io::stdin());
    let bytes = match config {
        Some(path) => match tokio::fs::read(path).await {
            Ok(bytes) => bytes,
            Err(error) => return Err(CliError::Io(error)),
        },
        None if config_stdin => {
            let mut bytes = Vec::new();
            if let Err(error) = input.read_until(b'\n', &mut bytes).await {
                return Err(CliError::Io(error));
            }
            bytes
        }
        None => Vec::new(),
    };
    let config: CacheProxyConfig = match serde_json::from_slice(&bytes) {
        Ok(config) => config,
        Err(error) => return Err(CliError::Json(error)),
    };
    let service: Arc<dyn ProxyControl> = Arc::new(VmProxy::new(config, network));
    let listener = match TcpListener::bind(listen).await {
        Ok(listener) => listener,
        Err(error) => return Err(CliError::Io(error)),
    };
    tracing::info!(%listen, "VM cache proxy listening");
    if ready_stdout {
        println!("READY");
    }
    let control = service.clone();
    let updates = tokio::spawn(async move {
        if config_stdin && update(input, control, ready_stdout).await.is_err() {
            tracing::warn!("VM proxy credential update unavailable");
        }
    });
    let result = match axum::serve(listener, job_router(service)).await {
        Ok(()) => Ok(()),
        Err(error) => Err(CliError::Io(error)),
    };
    updates.abort();
    result
}

async fn update(
    mut input: BufReader<Stdin>,
    control: Arc<dyn ProxyControl>,
    acknowledge: bool,
) -> Result<(), CliError> {
    loop {
        let mut bytes = Vec::new();
        match input.read_until(b'\n', &mut bytes).await {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(error) => return Err(CliError::Io(error)),
        }
        let update: CacheProxyUpdate = match serde_json::from_slice(&bytes) {
            Ok(update) => update,
            Err(error) => return Err(CliError::Json(error)),
        };
        control.replace_connection(update.connection).await;
        if acknowledge {
            println!("UPDATED {}", update.sequence);
        }
    }
}
