//! Production composition for the Linux machine agent.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::Parser;
use tokio::sync::{Mutex, mpsc};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;

use crate::application_cache;
use crate::application_endpoint::{self, ConfiguredAction};
use crate::application_setup::{
    InvocationError, InvocationSetup, LoadedInvocation, PreparedInvocation,
};
use crate::application_uninstall;
use crate::area::AreaFiles;
use crate::area_files::LinuxAreaFiles;
use crate::boundaries::DockerDisks;
use crate::boundaries::{Host, Service, Vms};
use crate::cli::{Action, Cli};
use crate::controller::{Agent, Controller};
use crate::docker::LinuxDockers;
use crate::endpoint::{CommandEndpoint, EndpointSettings};
use crate::endpoint_files::ControlEndpointFiles;
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::ReconnectingGateway;
use crate::hardware::LinuxHost;
use crate::host_composition::services;
use crate::host_file_ops::directory;
use crate::host_paths::{AREA, CONTROL, tools};
use crate::host_preparation::{FinalUninstall, uninstall_inventory};
use crate::host_system::LinuxHostSystem;
use crate::identity_store::IdentityStore;
use crate::installation::{Install, Installer};
use crate::layout::Layout;
use crate::logging::subscriber;
use crate::maintenance::{HourlyMaintenance, TokioDelay};
use crate::network::LinuxNetwork;
use crate::persistence::FileStore;
use crate::preparation::{Uninstall, stage_error};
use crate::process::{Commands, LinuxCommands};
use crate::runtime::{AgentDaemon, Daemon};
use crate::runtime_ownership::{CliRuntimeWaitNotice, FileRuntimeClaims, RuntimeClaims};
use crate::startup::{AgentStartup, Startup, StartupState};
use crate::state::Mode;
use crate::storage_sizing::{DEFAULT_IMAGE_BYTES, setting};
use crate::system_service::SystemService;
use crate::updates::AgentUpdates;
use crate::vm_files::LinuxVmFiles;
use crate::vm_root::{LinuxRootFiles, ReflinkRoot};
use crate::vms::LinuxVms;
use crate::vsock::LinuxGuests;
use crate::workload_space::{MachineWorkloadSpace, WorkloadSpace};

pub(crate) async fn main() -> Result<()> {
    let cli = Cli::parse();
    let directives = std::env::var("RUST_LOG").unwrap_or_default();
    subscriber(BoxMakeWriter::new(std::io::stderr), &directives)
        .try_init()
        .context("initialize safe JSON logging")?;
    execute(cli.action()).await
}

async fn execute(action: Action) -> Result<()> {
    let endpoint_files = Arc::new(ControlEndpointFiles);
    let endpoints: Arc<dyn EndpointSettings> = Arc::new(CommandEndpoint {
        files: endpoint_files.clone(),
    });
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
    let store = Arc::new(IdentityStore {
        store: Arc::new(FileStore {
            directory: PathBuf::from(CONTROL),
        }),
        endpoints: endpoint_files,
    });
    let runtime: Arc<dyn RuntimeClaims> = Arc::new(FileRuntimeClaims {
        control: PathBuf::from(CONTROL),
        notice: Arc::new(CliRuntimeWaitNotice),
    });
    let setup: Arc<dyn InvocationSetup> = Arc::new(LoadedInvocation {
        endpoints,
        service: service.clone(),
        identity: store.clone(),
        runtime: runtime.clone(),
    });
    let PreparedInvocation {
        action,
        state,
        ownership,
    } = match setup.prepare(action) {
        Ok(invocation) => invocation,
        Err(InvocationError::State(error)) => {
            return Err(stage_error(
                error,
                "machine identity state",
                "Restore /var/lib/ciproxy/state.json from a root-only backup. Keep existing job data.",
            ));
        }
        Err(InvocationError::Setup(error)) => return Err(error),
    };
    let inventory = if matches!(action, ConfiguredAction::Uninstall) {
        uninstall_inventory(files.as_ref())?
    } else {
        files.inspect()?
    };
    let image_budget = inventory
        .record
        .map_or(requested_budget.unwrap_or(DEFAULT_IMAGE_BYTES), |record| {
            record.image_budget
        });
    let preclaimed = ownership.map(|ownership| ownership.runtime());
    let image_services =
        crate::application_images::prepare(&layout, image_budget, runtime, preclaimed.clone())?;
    let space = image_services.space;
    let images = image_services.images;
    let identity = state.identity.clone();
    let removing = state.mode == Mode::Removed;
    let commands: Arc<dyn Commands> = Arc::new(LinuxCommands);
    let cache_services = application_cache::prepare(commands.clone(), identity.as_ref())?;
    let cache = cache_services.cache.clone();
    let (guest_tx, guest_rx) = mpsc::channel(256);
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
    let allocation = Arc::new(Mutex::new(()));
    let state = Arc::new(Mutex::new(state));
    let workload_space: Arc<dyn WorkloadSpace> = Arc::new(MachineWorkloadSpace {
        state: state.clone(),
        host: host.clone(),
        cache: cache.clone(),
    });
    let agent: Arc<dyn Agent> = Arc::new(Controller {
        allocation: allocation.clone(),
        reporting_initialized: Mutex::new(false),
        network_cleanup: Mutex::new(()),
        control_connected: AtomicBool::new(false),
        state,
        store: store.clone(),
        host: host.clone(),
        vms,
        images: images.clone(),
        cache: cache.clone(),
        space: workload_space.clone(),
        image_budget,
        dockers: Some(Arc::new(LinuxDockers {
            layout: layout.clone(),
            commands: commands.clone(),
            allocation,
            space: workload_space,
            image_budget,
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
                store,
                host,
                service,
                connection,
                agent,
                preparation,
            });
            let outcome = installer.install(token).await?;
            println!("{}", outcome.message());
        }
        ConfiguredAction::Run { endpoint } => {
            let connection = application_endpoint::retained_connection(endpoint.as_ref());
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
            daemon.run(identity, guest_rx, removing).await?;
        }
        ConfiguredAction::Uninstall => {
            let Some(runtime) = preclaimed else {
                return Err(Error::ImageOwnership);
            };
            let uninstall: Arc<dyn Uninstall> = Arc::new(FinalUninstall {
                _runtime: runtime,
                files,
                images,
                preparation,
                service,
                store,
                agent: agent.clone(),
            });
            application_uninstall::execute(uninstall, agent, guest_rx).await?;
        }
    }
    Ok(())
}
