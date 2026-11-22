//! One serialized own-machine execution domain and restart reconciliation.

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_gateway::gateway::MachineGateway;
use ciproxy_gateway::types::Session;
use ciproxy_github::assignments::JobAssignments;
use ciproxy_github::installation_state::InstallationApi;
use ciproxy_github::registrations::RunnerRegistrations;
use ciproxy_github::runtime::Runtime;
use ciproxy_overflow::api::Overflow;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runtime_results::BoatRunnerGuard;
use ciproxy_store_interface::store::Store;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::boat_authority::BoatRegistrationRetirement;
use crate::error::{Error, ErrorResultExt, Result};
use crate::image_signing::ImageSigning;
use crate::runtime_tasks::RuntimeTasks;
use crate::takeout::TakeoutCheck;

/// Runtime entry points shared by HTTP events, listeners, and periodic repair.
#[async_trait]
pub trait OwnMachines: Send + Sync {
    /// Dispatch retirement of accessible idle registrations after an Admin removal.
    async fn machine_removed(&self, workspace_id: Uuid, machine_id: Uuid) -> Result<()>;
    /// Queue an authenticated event without waiting for runtime processing.
    fn enqueue_agent(&self, session: Session, message: AgentMessage);
    /// Queue saved machine removal without waiting for registration access.
    fn enqueue_machine_removed(&self, workspace_id: Uuid, machine_id: Uuid);
    /// Cancel owned work if the runtime supervisor exits or is cancelled.
    fn cancel_tasks(&self);
    /// Stop and join all owned remote work during process shutdown.
    async fn shutdown(&self) -> Result<()>;
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
    /// GitHub-first recovery of interrupted Boat registrations.
    pub boat_retirement: Option<Arc<dyn BoatRegistrationRetirement>>,
    /// Shared optional Boat service, configured only with the deployed image snapshot.
    pub overflow: Option<Arc<dyn Overflow>>,
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
    /// Released images, at most one for each platform.
    pub images: Vec<ImageOffer>,
    pub(crate) operation: Arc<Mutex<()>>,
    pub(crate) tasks: Weak<RuntimeTasks>,
    task_owner: Option<Arc<RuntimeTasks>>,
    pub(crate) boat_operations: Arc<Mutex<HashMap<Uuid, Arc<Mutex<()>>>>>,
    pub(crate) image_requests: Arc<Mutex<HashMap<(Uuid, Uuid), ImageRequest>>>,
    pub(crate) boat_answers: Arc<Mutex<HashMap<Uuid, (u64, BoatRunnerGuard)>>>,
    pub(crate) takeout_checks: Arc<Mutex<HashMap<Uuid, TakeoutCheck>>>,
}

/// One released image that the runtime offers to the machines of its platform.
#[derive(Clone)]
pub struct ImageOffer {
    /// Release facts with the unsigned artifact URL.
    pub download: ImageDownload,
    /// Signs a fresh URL for each send. Without it, the URL goes out as it is.
    pub signing: Option<Arc<dyn ImageSigning>>,
}

impl ImageOffer {
    /// Offer a release whose URL needs no signature.
    pub fn unsigned(download: ImageDownload) -> Self {
        Self {
            download,
            signing: None,
        }
    }
}

pub(crate) struct ImageRequest {
    pub(crate) image_id: ImageId,
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
        images: impl IntoIterator<Item = ImageDownload>,
        installations: Arc<dyn InstallationApi>,
    ) -> Self {
        let tasks = RuntimeTasks::new();
        Self {
            boat_retirement: None,
            overflow: None,
            store,
            gateway,
            registrations,
            assignments,
            clock,
            images: images.into_iter().map(ImageOffer::unsigned).collect(),
            installations,
            operation: Arc::new(Mutex::new(())),
            tasks: Arc::downgrade(&tasks),
            task_owner: Some(tasks),
            boat_operations: Arc::new(Mutex::new(HashMap::new())),
            image_requests: Arc::new(Mutex::new(HashMap::new())),
            takeout_checks: Arc::new(Mutex::new(HashMap::new())),
            boat_answers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    /// Wait until queued events and all remote tasks finish, returning a joined error.
    pub async fn quiesce(&self) -> Result<()> {
        self.task_scope()?.quiesce().await
    }

    pub(crate) fn task_scope(&self) -> Result<Arc<RuntimeTasks>> {
        self.tasks.upgrade().ok_or(Error::StateChanged)
    }

    pub(crate) fn worker(&self) -> Self {
        Self {
            boat_retirement: self.boat_retirement.clone(),
            overflow: self.overflow.clone(),
            store: self.store.clone(),
            gateway: self.gateway.clone(),
            registrations: self.registrations.clone(),
            assignments: self.assignments.clone(),
            clock: self.clock.clone(),
            installations: self.installations.clone(),
            images: self.images.clone(),
            operation: self.operation.clone(),
            image_requests: self.image_requests.clone(),
            takeout_checks: self.takeout_checks.clone(),
            boat_answers: self.boat_answers.clone(),
            tasks: self.tasks.clone(),
            task_owner: None,
            boat_operations: self.boat_operations.clone(),
        }
    }

    /// The release that the machines of this platform get.
    pub(crate) fn offer(&self, platform: Platform) -> Option<&ImageOffer> {
        self.images
            .iter()
            .find(|offer| offer.download.platform == platform)
    }

    /// Root sizing of Linux job VMs and Boat sandboxes, from the Linux release.
    pub(crate) fn root_sizing(&self) -> RootSizing {
        self.offer(Platform::LinuxX64)
            .map_or_else(RootSizing::default, |offer| RootSizing {
                image_root_bytes: offer.download.root_disk_bytes,
            })
    }

    pub(crate) async fn load(&self, workspace_id: Uuid) -> Result<RoutingSnapshot> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.own_routing_snapshot(workspace_id))
            .await
            .internal()?
            .internal()
    }

    /// Keep required local cleanup independent of optional provider work.
    pub(crate) async fn purge_boat_registration_metadata(&self, workspace_id: Uuid) -> Result<()> {
        let store = self.store.clone();
        let at = self.clock.now();
        tokio::task::spawn_blocking(move || {
            store.purge_terminal_boat_runner_intents(workspace_id, at)
        })
        .await
        .internal()?
        .internal()?;
        Ok(())
    }

    /// Load current admission while retaining every organization's completion context.
    pub(crate) async fn runnable_organizations(
        &self,
        input: &RoutingSnapshot,
    ) -> Result<Vec<Uuid>> {
        let store = self.store.clone();
        let workspace_id = input.workspace_id;
        let organizations: Vec<_> = input
            .organizations
            .iter()
            .map(|organization| organization.id)
            .collect();
        tokio::task::spawn_blocking(move || {
            let mut runnable = Vec::new();
            for organization_id in organizations {
                if store
                    .organization_runnable(workspace_id, organization_id)
                    .internal()?
                {
                    runnable.push(organization_id);
                }
            }
            Ok(runnable)
        })
        .await
        .internal()?
    }
}

impl Drop for MachineRuntime {
    fn drop(&mut self) {
        if self.task_owner.is_some() {
            tracing::debug!("runtime owner stopped; owned tasks will stop");
        }
    }
}
