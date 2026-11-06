//! JIT runner registration and race-safe removal outcomes.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::scale_sets::ScaleSetState;

use crate::error::{Error, Result};
use crate::persistence::session::{ListenerTarget, SessionStore};
use crate::scale_sets::{JitRegistration, ScaleSetFactory};

/// The result of a requested GitHub registration removal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Removal {
    /// The registration was deleted or was already absent.
    Removed,
    /// GitHub assigned a job, so the VM and its running work must remain.
    Busy,
    /// Removal revoked App access, so the registration must stay on GitHub.
    AccessGone,
}

/// The runtime's JIT configuration boundary for one ephemeral runner.
#[async_trait]
pub trait RunnerRegistrations: Send + Sync {
    /// Register one runner after the orchestrator reserves its destination.
    async fn create(&self, target: ListenerTarget, runner_name: &str) -> Result<JitRegistration>;
    /// Remove an idle/ended registration before the orchestrator frees its VM.
    async fn remove(&self, target: ListenerTarget, github_runner_id: i64) -> Result<Removal>;
}

/// The JIT service, consumed through dyn RunnerRegistrations.
pub struct RegistrationService {
    /// Tenant-scoped organization and scale-set context.
    pub store: Arc<dyn SessionStore>,
    /// Organization-scoped APIs with the control-plane App credentials.
    pub scale_sets: Arc<dyn ScaleSetFactory>,
}

#[async_trait]
impl RunnerRegistrations for RegistrationService {
    async fn create(&self, target: ListenerTarget, runner_name: &str) -> Result<JitRegistration> {
        let context = self.store.context(target).await?;
        if context.organization.app_state != AppState::Active
            || context.state != ScaleSetState::Active
        {
            return Err(Error::OrganizationInactive);
        }
        let api = self
            .scale_sets
            .for_organization(&context.organization)
            .await?;
        api.register(context.scale_set.github_id, runner_name).await
    }

    async fn remove(&self, target: ListenerTarget, github_runner_id: i64) -> Result<Removal> {
        let context = self.store.context(target).await?;
        if context.organization.app_state == AppState::Removed {
            return Ok(Removal::AccessGone);
        }
        let api = self
            .scale_sets
            .for_organization(&context.organization)
            .await?;
        match api.remove_runner(github_runner_id).await {
            Ok(()) => Ok(Removal::Removed),
            Err(Error::RunnerBusy) => Ok(Removal::Busy),
            Err(error) => Err(error),
        }
    }
}
