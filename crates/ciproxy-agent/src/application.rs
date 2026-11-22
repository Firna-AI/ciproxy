//! Production composition of the machine agent over the native platform.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::Parser;
use tokio::sync::Mutex;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::util::SubscriberInitExt;

use crate::application_endpoint::{self, ConfiguredAction};
use crate::application_setup::{
    InvocationError, InvocationSetup, LoadedInvocation, PreparedInvocation,
};
use crate::application_uninstall;
use crate::cli::{Action, Cli};
use crate::controller::{Agent, Controller};
use crate::endpoint::{CommandEndpoint, EndpointSettings};
use crate::endpoint_files::ControlEndpointFiles;
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::ReconnectingGateway;
use crate::host_paths::CONTROL;
use crate::identity_store::IdentityStore;
use crate::installation::{Install, Installer};
use crate::logging::subscriber;
use crate::maintenance::{HourlyMaintenance, TokioDelay};
use crate::persistence::FileStore;
use crate::platform::{Machine, MachineInput, NativePlatform, Platform, UninstallParts};
use crate::preparation::stage_error;
use crate::runtime::{AgentDaemon, Daemon};
use crate::runtime_ownership::{CliRuntimeWaitNotice, FileRuntimeClaims, RuntimeClaims};
use crate::startup::{AgentStartup, Startup, StartupState};
use crate::state::Mode;
use crate::workload_space::{MachineWorkloadSpace, WorkloadSpace};

pub(crate) async fn main() -> Result<()> {
    let cli = Cli::parse();
    let directives = std::env::var("RUST_LOG").unwrap_or_default();
    let action = cli.action();
    subscriber(log_writer(&action), &directives)
        .try_init()
        .context("initialize safe JSON logging")?;
    #[cfg(target_os = "macos")]
    match &action {
        Action::VmRun { local_id } => return crate::platform::run_vm_helper(local_id).await,
        Action::PrepareHost { with_service } => {
            return crate::platform::prepare_host(*with_service).await;
        }
        _ => {}
    }
    execute(NativePlatform::new(), action).await
}

/// On a Mac, the launchd jobs write the log file themselves, so the log
/// follows a rotation. Interactive commands and Linux use standard error.
fn log_writer(action: &Action) -> BoxMakeWriter {
    #[cfg(target_os = "macos")]
    if matches!(action, Action::Run | Action::VmRun { .. })
        && let Ok(log) = crate::macos::log_file::ReopeningLog::open(std::path::Path::new(
            crate::macos::paths::LOG,
        ))
    {
        return BoxMakeWriter::new(move || crate::macos::log_file::LogWriter(log.clone()));
    }
    let _ = action;
    BoxMakeWriter::new(std::io::stderr)
}

async fn execute<P: Platform>(platform: P, action: Action) -> Result<()> {
    let endpoint_files = Arc::new(ControlEndpointFiles);
    let endpoints: Arc<dyn EndpointSettings> = Arc::new(CommandEndpoint {
        files: endpoint_files.clone(),
    });
    let service = platform.service();
    let host = platform.host();
    host.require_root()?;
    let (preparation, prepared) = platform.prepare()?;
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
    let preclaimed = ownership.map(|ownership| ownership.runtime());
    let Machine {
        image_budget,
        images,
        cache,
        vms,
        guests: guest_rx,
        tasks: _tasks,
    } = platform.machine(
        prepared,
        MachineInput {
            uninstall: matches!(action, ConfiguredAction::Uninstall),
            claims: runtime,
            owner: preclaimed.clone(),
            identity: state.identity.as_ref(),
        },
    )?;
    let identity = state.identity.clone();
    let removing = state.mode == Mode::Removed;
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
        dockers: platform.dockers(allocation, workload_space, image_budget),
    });
    match action {
        ConfiguredAction::Install { token, endpoint } => {
            let connection = application_endpoint::connection(&endpoint);
            preparation.prepare().await?;
            if let Err(error) = platform.install_cache().await {
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
                    updates: platform.updates(),
                }),
            });
            daemon.run(identity, guest_rx, removing).await?;
        }
        ConfiguredAction::Uninstall => {
            let Some(runtime) = preclaimed else {
                return Err(Error::ImageOwnership);
            };
            let uninstall = platform.uninstall(UninstallParts {
                runtime,
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
