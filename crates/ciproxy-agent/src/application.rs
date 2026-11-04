//! Production composition for the Linux machine agent.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ciproxy_protocol::reports::REPORT_INTERVAL_SECONDS;
use clap::Parser;
use reqwest::Client;
use tokio::sync::{Mutex, mpsc};
use tokio::time::interval;

use crate::boundaries::{Cache, Connection, Host, Images, Service, Store, Vms};
use crate::cache::UnixCache;
use crate::cli::{Action, Cli};
use crate::connection::WebSocketConnection;
use crate::controller::{Agent, Controller};
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::ReconnectingGateway;
use crate::hardware::LinuxHost;
use crate::image_download::HttpsImageDownload;
use crate::image_store::LinuxImageStore;
use crate::images::ManagedImages;
use crate::installation::{Install, Installer, PendingWorkspaceResolver};
use crate::layout::Layout;
use crate::maintenance::{HourlyMaintenance, TokioDelay};
use crate::network::LinuxNetwork;
use crate::persistence::FileStore;
use crate::process::{Commands, LinuxCommands};
use crate::runtime::{AgentDaemon, Daemon};
use crate::state::Mode;
use crate::system_service::SystemService;
use crate::updates::AgentUpdates;
use crate::vm_files::LinuxVmFiles;
use crate::vms::LinuxVms;
use crate::vsock::LinuxGuests;

pub(crate) async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt().json().init();
    execute(cli.action()).await
}

async fn execute(action: Action) -> Result<()> {
    let service: Arc<dyn Service> = Arc::new(SystemService);
    let layout = Layout {
        data: PathBuf::from("/var/lib/ciproxy"),
        firecracker: PathBuf::from("/usr/local/bin/firecracker"),
        jailer: PathBuf::from("/usr/local/bin/jailer"),
    };
    let host: Arc<dyn Host> = Arc::new(LinuxHost {
        data_directory: layout.data.clone(),
    });
    host.require_root()?;
    if matches!(action, Action::Uninstall) {
        service.stop()?;
    }
    let store: Arc<dyn Store> = Arc::new(FileStore {
        directory: layout.data.clone(),
    });
    let state = store.load()?;
    let identity = state.identity.clone();
    let removing = state.mode == Mode::Removed;
    let commands: Arc<dyn Commands> = Arc::new(LinuxCommands);
    let cache: Arc<dyn Cache> = Arc::new(UnixCache {
        socket: PathBuf::from("/run/ciproxy-cache/control.sock"),
    });
    let images: Arc<dyn Images> = Arc::new(ManagedImages::new(
        Arc::new(LinuxImageStore::new(layout.clone())),
        Arc::new(HttpsImageDownload {
            client: Client::builder()
                .https_only(true)
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(3600))
                .build()
                .context("configure image downloads")?,
        }),
    ));
    let (guest_tx, mut guest_rx) = mpsc::channel(256);
    let guests = Arc::new(LinuxGuests::new(
        layout.clone(),
        commands.clone(),
        cache.clone(),
        guest_tx,
    ));
    let vms: Arc<dyn Vms> = Arc::new(LinuxVms {
        layout: layout.clone(),
        commands: commands.clone(),
        network: Arc::new(LinuxNetwork {
            commands: commands.clone(),
        }),
        files: Arc::new(LinuxVmFiles {
            layout,
            commands: commands.clone(),
        }),
        guests,
        operation: Mutex::new(()),
    });
    let agent: Arc<dyn Agent> = Arc::new(Controller {
        state: Mutex::new(state),
        store: store.clone(),
        host: host.clone(),
        vms,
        images: images.clone(),
        cache,
    });
    let connection: Arc<dyn Connection> = Arc::new(WebSocketConnection::new(
        std::env::var("CIPROXY_AGENT_URL")
            .unwrap_or_else(|_| "wss://app.ciproxy.dev/agent/".to_owned()),
    ));
    match action {
        Action::Install(token) => {
            let installer: Arc<dyn Install> = Arc::new(Installer {
                store,
                host,
                service,
                connection,
                agent,
                resolver: Arc::new(PendingWorkspaceResolver),
            });
            installer.install(token).await?;
            println!("ciproxy agent installed");
        }
        Action::Run => {
            images.recover()?;
            if removing && agent.finished().await {
                return Ok(());
            }
            let identity = identity.ok_or(Error::NotInstalled)?;
            let delay = Arc::new(TokioDelay);
            let daemon: Arc<dyn Daemon> = Arc::new(AgentDaemon {
                agent: agent.clone(),
                connection: connection.clone(),
                gateway: Arc::new(ReconnectingGateway {
                    connection,
                    agent: agent.clone(),
                    delay: delay.clone(),
                }),
                maintenance: Arc::new(HourlyMaintenance {
                    delay,
                    updates: Arc::new(AgentUpdates {
                        commands,
                        release_server: std::env::var("CIPROXY_RELEASE_SERVER_URL")
                            .unwrap_or_else(|_| "https://get.ciproxy.dev".to_owned()),
                    }),
                }),
            });
            daemon.run(identity, guest_rx, removing).await?;
            let mut state = store.load()?;
            state.identity = None;
            store.save(&state)?;
        }
        Action::Uninstall => {
            images.recover()?;
            agent.restore().await?;
            agent.remove().await?;
            let mut timer = interval(Duration::from_secs(REPORT_INTERVAL_SECONDS));
            while !agent.finished().await {
                let result = tokio::select! {
                    event = guest_rx.recv() => {
                        let event = event.ok_or(Error::Disconnected)?;
                        agent.guest(&event.runner_id.0, event.message).await.map(|_| ())
                    }
                    _ = timer.tick() => agent.reconcile().await,
                };
                if let Err(error) = result {
                    tracing::warn!(error = %error, "uninstall is waiting for job and VM cleanup");
                }
            }
            images.purge()?;
            store.clear()?;
            service.uninstall()?;
            println!("ciproxy agent uninstalled");
        }
    }
    Ok(())
}
