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
pub enum RegistrationRetirement {
    /// The registration was deleted or was already absent.
    Removed {
        /// The exact runner whose removal or absence GitHub confirmed.
        id: i64,
    },
    /// GitHub assigned a job, so the VM and its running work must remain.
    Busy {
        /// The exact runner that GitHub reports busy.
        id: i64,
    },
    /// A successful live unique-name lookup found no registration.
    Absent,
}

/// The runtime's JIT configuration boundary for one ephemeral runner.
#[unimock::unimock(api = RunnerRegistrationsMock)]
#[async_trait]
pub trait RunnerRegistrations: Send + Sync {
    /// Register one runner after the orchestrator reserves its destination.
    async fn create(&self, target: ListenerTarget, runner_name: &str) -> Result<JitRegistration>;
    /// Confirm retirement through live lookup/removal, or preserve a busy runner.
    async fn retire(
        &self,
        target: ListenerTarget,
        runner_name: &str,
        saved_id: Option<i64>,
    ) -> Result<RegistrationRetirement>;
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

    async fn retire(
        &self,
        target: ListenerTarget,
        runner_name: &str,
        saved_id: Option<i64>,
    ) -> Result<RegistrationRetirement> {
        let context = self.store.context(target).await?;
        let api = self
            .scale_sets
            .for_organization(&context.organization)
            .await?;
        let id = match saved_id {
            Some(id) => Some(id),
            None => api.find_runner(runner_name).await?,
        };
        let Some(id) = id else {
            return Ok(RegistrationRetirement::Absent);
        };
        match api.remove_runner(id).await {
            Ok(()) => Ok(RegistrationRetirement::Removed { id }),
            Err(Error::RunnerBusy) => Ok(RegistrationRetirement::Busy { id }),
            Err(error) => Err(error),
        }
    }
}
