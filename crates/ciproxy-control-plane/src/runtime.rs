//! One serialized own-machine execution domain and restart reconciliation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_gateway::gateway::MachineGateway;
use ciproxy_github::assignments::JobAssignments;
use ciproxy_github::installation_state::InstallationApi;
use ciproxy_github::registrations::RunnerRegistrations;
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use ciproxy_store_interface::store::Store;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::image_signing::ImageSigning;
use crate::snapshot::build;
use ciproxy_store_interface::own_runners::RoutingSnapshot;

/// Runtime entry points shared by HTTP events, listeners, and periodic repair.
#[async_trait]
pub trait OwnMachines: Send + Sync {
    /// Remove accessible idle registrations synchronously with an Admin removal.
    async fn machine_removed(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<()>;
    /// Reconcile every retained workspace independently after restart or a tick.
    async fn tick(&self) -> Result<()>;
    /// Route a saved job update through its durable GitHub runner mapping.
    async fn job(&self, workspace_id: Uuid, job: GithubJob) -> Result<()>;
    /// Consume a lifecycle event under authenticated machine ownership.
    async fn agent(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: AgentMessage,
    ) -> Result<()>;
    /// Apply installation changes while preserving jobs already in progress.
    async fn lifecycle(&self, event: LifecycleEvent) -> Result<()>;
}

/// Dependencies assembled once at the hosted composition root.
pub struct MachineRuntime {
    /// Tenant-scoped durable persistence.
    pub store: Arc<dyn Store>,
    /// The shared owner of machine WebSockets and command routing.
    pub gateway: Arc<dyn MachineGateway>,
    /// GitHub registration and safe removal operations.
    pub registrations: Arc<dyn RunnerRegistrations>,
    /// Trusted API facts with expected signed-token identities.
    pub assignments: Arc<dyn JobAssignments>,
    /// Injected clock and identities.
    pub clock: Arc<dyn Runtime>,
    /// Exact App-JWT installation lookup for pending take-out authority.
    pub installations: Arc<dyn InstallationApi>,
    /// Authenticated release download, refreshed by release composition.
    pub image: Option<ImageDownload>,
    /// Production CDN signer, supplied only at the hosted composition root.
    pub image_signing: Option<Arc<dyn ImageSigning>>,
    pub(crate) operation: Mutex<()>,
    pub(crate) image_requests: Mutex<HashMap<(Uuid, Uuid), ImageRequest>>,
}

pub(crate) struct ImageRequest {
    pub(crate) image_id: ciproxy_protocol::identifiers::ImageId,
    pub(crate) session_id: Uuid,
    pub(crate) command_id: Uuid,
}

impl MachineRuntime {
    /// Construct the execution service from explicit side-effect boundaries.
    pub fn new(
        store: Arc<dyn Store>,
        gateway: Arc<dyn MachineGateway>,
        registrations: Arc<dyn RunnerRegistrations>,
        assignments: Arc<dyn JobAssignments>,
        clock: Arc<dyn Runtime>,
        image: Option<ImageDownload>,
        installations: Arc<dyn InstallationApi>,
    ) -> Self {
        Self {
            store,
            gateway,
            registrations,
            assignments,
            clock,
            image,
            installations,
            image_signing: None,
            operation: Mutex::new(()),
            image_requests: Mutex::new(HashMap::new()),
        }
    }
    pub(crate) async fn load(&self, workspace_id: Uuid) -> Result<RoutingSnapshot> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.own_routing_snapshot(workspace_id))
            .await
            .internal()?
            .internal()
    }

    /// Run ordinary best-effort routing under this workspace execution lease.
    pub(crate) async fn reconcile_workspace(&self, workspace_id: Uuid) -> Result<()> {
        let store = self.store.clone();
        let Some(mut lease) =
            tokio::task::spawn_blocking(move || store.try_lock_own_runners(workspace_id))
                .await
                .internal()?
                .internal()?
        else {
            return Ok(());
        };
        let mut input = self.load(workspace_id).await?;
        self.reconcile_reports(&input).await?;
        input = self.load(workspace_id).await?;
        self.recover_stops(&input, &mut *lease).await?;
        for job in &input.jobs {
            self.route_job(&input, job).await?;
        }
        self.ensure_image(&input).await?;
        for machine in &input.machines {
            if let Err(error) = self.send_docker_budget(&input, machine.machine.id).await {
                tracing::warn!(machine=%machine.machine.id,%error,"Docker policy update will retry");
            }
        }
        let root_sizing = self
            .image
            .as_ref()
            .map_or_else(RootSizing::default, |image| RootSizing {
                image_root_bytes: image.root_disk_bytes,
            });
        let mut snapshot = build(&input, self.clock.now(), root_sizing);
        for machine in &mut snapshot.machines {
            let confirmed = input
                .machines
                .iter()
                .find(|candidate| candidate.machine.id.to_string() == machine.id.0)
                .and_then(|candidate| candidate.report.as_ref())
                .is_some_and(|report| report.report.images.selected().is_some());
            machine.connected &= confirmed;
        }
        for action in ciproxy_scaler::decision::decide(&snapshot) {
            lease.check_connection().internal()?;
            match self.apply(&input, action).await {
                Ok(()) => {}
                Err(Error::StateChanged) => return Err(Error::StateChanged),
                Err(error) => {
                    tracing::warn!(%workspace_id,%error,"runner action will retry; independent actions continue")
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl OwnMachines for MachineRuntime {
    async fn machine_removed(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<()> {
        let _operation = self.operation.lock().await;
        let input = self.load(workspace_id).await?;
        for runner in &input.runners {
            if matches!(runner.runner.place,ciproxy_store_interface::runners::RunnerPlace::Machine{machine_id:owner,..}if owner==machine_id)
                && runner.runner.state != ciproxy_store_interface::runners::RunnerState::Stopped
            {
                self.stop(&input, runner).await?;
            }
        }
        Ok(())
    }
    async fn tick(&self) -> Result<()> {
        let _operation = self.operation.lock().await;
        let store = self.store.clone();
        let workspaces = tokio::task::spawn_blocking(move || store.routing_workspaces())
            .await
            .internal()?
            .internal()?;
        for workspace_id in workspaces {
            if let Err(error) = self.reconcile_workspace(workspace_id).await {
                tracing::warn!(%workspace_id, %error, "own-machine reconciliation will retry");
            }
        }
        Ok(())
    }

    async fn job(&self, workspace_id: Uuid, job: GithubJob) -> Result<()> {
        let _operation = self.operation.lock().await;
        let input = self.load(workspace_id).await?;
        self.route_job(&input, &job).await
    }

    async fn agent(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: AgentMessage,
    ) -> Result<()> {
        let _operation = self.operation.lock().await;
        self.agent_event(workspace_id, machine_id, message).await
    }

    async fn lifecycle(&self, event: LifecycleEvent) -> Result<()> {
        let workspace_id = event.organization().workspace_id;
        let _operation = self.operation.lock().await;
        if let LifecycleEvent::Removed {
            organization,
            cause,
            ..
        } = &event
        {
            return match cause {
                RemovalCause::Owner => self.owner_removed(organization).await,
                RemovalCause::WorkspaceTakeout => self.takeout(organization).await,
            };
        }
        self.reconcile_workspace(workspace_id).await
    }
}
