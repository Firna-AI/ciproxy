//! Production composition for the Linux machine agent.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use ciproxy_protocol::reports::REPORT_INTERVAL_SECONDS;
use clap::Parser;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::{Mutex, mpsc};
use tokio::time::interval;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;

use crate::application_cache;
use crate::application_endpoint::{self, ConfiguredAction};
use crate::area::AreaFiles;
use crate::area_files::LinuxAreaFiles;
use crate::boundaries::{Host, Service, Store, Vms};
use crate::cli::{Action, Cli};
use crate::controller::{Agent, Controller};
use crate::docker::{DockerDisks, LinuxDockers};
use crate::endpoint::{EndpointSettings, SavedEndpoint};
use crate::endpoint_files::LinuxEndpointFiles;
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::ReconnectingGateway;
use crate::hardware::LinuxHost;
use crate::host_composition::services;
use crate::host_file_ops::directory;
use crate::host_paths::{AREA, CONTROL, tools};
use crate::host_preparation::{
    FinalUninstall, Uninstall, UninstallState, stage_error, uninstall_inventory,
};
use crate::host_system::LinuxHostSystem;
use crate::installation::{Install, Installer};
use crate::layout::Layout;
use crate::logging::subscriber;
use crate::maintenance::{HourlyMaintenance, TokioDelay};
use crate::network::LinuxNetwork;
use crate::persistence::FileStore;
use crate::process::{Commands, LinuxCommands};
use crate::runtime::{AgentDaemon, Daemon};
use crate::startup::{AgentStartup, Startup, StartupState};
use crate::state::Mode;
use crate::storage_sizing::{DEFAULT_IMAGE_BYTES, setting};
use crate::system_service::SystemService;
use crate::updates::AgentUpdates;
use crate::vm_files::LinuxVmFiles;
use crate::vm_root::{LinuxRootFiles, ReflinkRoot};
use crate::vms::LinuxVms;
use crate::vsock::LinuxGuests;

pub(crate) async fn main() -> Result<()> {
    let cli = Cli::parse();
    let directives = std::env::var("RUST_LOG").unwrap_or_default();
    subscriber(BoxMakeWriter::new(std::io::stderr), &directives)
        .try_init()
        .context("initialize safe JSON logging")?;
    execute(cli.action()).await
}

async fn execute(action: Action) -> Result<()> {
    let endpoints: Arc<dyn EndpointSettings> = Arc::new(SavedEndpoint {
        files: Arc::new(LinuxEndpointFiles),
    });
    let action = application_endpoint::configure(action, endpoints.as_ref())?;
    let service: Arc<dyn Service> = Arc::new(SystemService);
    let layout = Layout {
        data: PathBuf::from(AREA),
        firecracker: tools().join("firecracker"),
        jailer: tools().join("jailer"),
    };
    let files: Arc<dyn AreaFiles> = Arc::new(LinuxAreaFiles::new());
    let system = Arc::new(LinuxHostSystem);
    let host: Arc<dyn Host> = Arc::new(LinuxHost {
        system: system.clone(),
        area: files.clone(),
    });
    host.require_root()?;
    if let Err(error) = directory(Path::new(CONTROL)) {
        return Err(stage_error(
            error,
            "control directory",
            "Restore root ownership and write access to /var/lib/ciproxy. Then rerun install.",
        ));
    }
    let requested = match std::env::var("CIPROXY_IMAGE_BUDGET_BYTES") {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(Error::Preparation {
                component: "image budget setting",
                repair: "Set CIPROXY_IMAGE_BUDGET_BYTES to an integer byte count.",
            });
        }
    };
    let requested_budget = setting(requested.as_deref())?;
    let services = services(files.clone(), system, requested_budget)?;
    let preparation = services.preparation;
    if matches!(action, ConfiguredAction::Uninstall { .. }) {
        service.stop()?;
    }
    let store: Arc<dyn Store> = Arc::new(FileStore {
        directory: PathBuf::from(CONTROL),
    });
    let state = match store.load() {
        Ok(state) => state,
        Err(error) => {
            return Err(stage_error(
                error,
                "machine identity state",
                "Restore /var/lib/ciproxy/state.json from a root-only backup. Keep existing job data.",
            ));
        }
    };
    let inventory = if matches!(action, ConfiguredAction::Uninstall { .. }) {
        uninstall_inventory(files.as_ref())?
    } else {
        files.inspect()?
    };
    let image_budget = inventory
        .record
        .map_or(requested_budget.unwrap_or(DEFAULT_IMAGE_BYTES), |record| {
            record.image_budget
        });
    let image_services = crate::application_images::prepare(&layout, image_budget)?;
    let space = image_services.space;
    let images = image_services.images;
    let identity = state.identity.clone();
    let removing = state.mode == Mode::Removed;
    let commands: Arc<dyn Commands> = Arc::new(LinuxCommands);
    let cache_services =
        application_cache::prepare(commands.clone(), action.endpoint(), identity.as_ref())?;
    let cache = cache_services.cache.clone();
    let (guest_tx, mut guest_rx) = mpsc::channel(256);
    let guests = Arc::new(LinuxGuests::new(layout.clone(), commands.clone(), guest_tx));
    let vms: Arc<dyn Vms> = Arc::new(LinuxVms {
        tools: services.tools,
        layout: layout.clone(),
        commands: commands.clone(),
        network: Arc::new(LinuxNetwork::new(
            commands.clone(),
            services.network_packages,
        )),
        files: Arc::new(LinuxVmFiles {
            layout: layout.clone(),
            root: Arc::new(ReflinkRoot {
                files: Arc::new(LinuxRootFiles),
                commands: commands.clone(),
            }),
            space,
        }),
        guests,
        operation: Mutex::new(()),
    });
    let agent: Arc<dyn Agent> = Arc::new(Controller {
        allocation: Mutex::new(()),
        reporting_initialized: Mutex::new(false),
        network_cleanup: Mutex::new(()),
        control_connected: AtomicBool::new(false),
        state: Mutex::new(state),
        store: store.clone(),
        host: host.clone(),
        vms,
        images: images.clone(),
        cache: cache.clone(),
        image_budget,
        dockers: Some(Arc::new(LinuxDockers {
            layout: layout.clone(),
            commands: commands.clone(),
            cache,
        }) as Arc<dyn DockerDisks>),
    });
    match action {
        ConfiguredAction::Install { token, endpoint } => {
            let connection = application_endpoint::connection(&endpoint);
            preparation.prepare().await?;
            let cache_service = application_cache::installer(commands.clone());
            if let Err(error) = cache_service.install().await {
                tracing::warn!(%error, "machine cache install unavailable; GitHub fallback remains available");
            }
            let installer: Arc<dyn Install> = Arc::new(Installer {
                endpoint,
                endpoints,
                store,
                host,
                service,
                connection,
                agent,
                preparation,
            });
            installer.install(token).await?;
            println!("ciproxy agent installed");
        }
        ConfiguredAction::Run { endpoint } => {
            let connection = application_endpoint::connection(&endpoint);
            let startup: Arc<dyn Startup> = Arc::new(AgentStartup {
                agent: agent.clone(),
                preparation,
                images,
            });
            let StartupState::Run(identity) = startup.prepare(identity, removing).await? else {
                return Ok(());
            };
            let delay = Arc::new(TokioDelay);
            let daemon: Arc<dyn Daemon> = Arc::new(AgentDaemon {
                agent: agent.clone(),
                connection: connection.clone(),
                gateway: Arc::new(ReconnectingGateway {
                    connection,
                    agent: agent.clone(),
                    delay: delay.clone(),
                }),
                report_delay: delay.clone(),
                maintenance: Arc::new(HourlyMaintenance {
                    delay,
                    updates: Arc::new(AgentUpdates {
                        commands,
                        release_server: std::env::var("CIPROXY_RELEASE_SERVER_URL")
                            .unwrap_or_else(|_| "https://get.ciproxy.dev".to_owned()),
                    }),
                }),
            });
            let mut signal =
                signal(SignalKind::user_defined1()).context("listen for local uninstall")?;
            let (remove, removal) = mpsc::channel(1);
            let input = tokio::spawn(async move {
                while signal.recv().await.is_some() {
                    if remove.send(()).await.is_err() {
                        return;
                    }
                }
            });
            let result = daemon.run(identity, guest_rx, removing, removal).await;
            input.abort();
            let _ = input.await;
            result?;
        }
        ConfiguredAction::Uninstall { .. } => {
            let uninstall: Arc<dyn Uninstall> = Arc::new(FinalUninstall {
                files,
                images,
                preparation,
                service,
                store,
                agent: agent.clone(),
            });
            if uninstall.start().await? == UninstallState::Complete {
                println!("ciproxy agent uninstalled");
                return Ok(());
            }
            let mut timer = interval(Duration::from_secs(REPORT_INTERVAL_SECONDS));
            loop {
                match uninstall.finish().await {
                    Ok(()) => break,
                    Err(Error::Draining) => {}
                    Err(error) => {
                        tracing::warn!(%error, "Uninstall cleanup will retry automatically. Stop this command before running uninstall again.")
                    }
                }
                let result = tokio::select! {
                    event = guest_rx.recv() => {
                        let event = event.ok_or(Error::Disconnected)?;
                        if event.configuration_lost {
                            agent.configuration_lost(&event.runner_id.0).await?;
                        }
                        agent.guest(event.provenance, &event.runner_id.0, event.message).await.map(|_| ())
                    }
                    _ = timer.tick() => agent.reconcile().await,
                };
                if let Err(error) = result {
                    tracing::warn!(error = %error, "uninstall is waiting for job and VM cleanup");
                }
            }
            println!("ciproxy agent uninstalled");
        }
    }
    Ok(())
}
