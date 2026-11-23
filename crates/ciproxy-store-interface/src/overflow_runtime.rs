//! Credential-free GitHub registration intents for recoverable overflow starts.

use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::overflow::provider::OverflowProvider;
use crate::overflow::store::OverflowSnapshot;
use crate::scale_sets::RegistrationTarget;

/// Stable identities saved before a one-use registration is requested.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OverflowRunnerIntent {
    /// The provider where this runner will start.
    #[serde(default)]
    pub provider: OverflowProvider,
    /// Exact remote registration target. Older saved JSON keeps this unknown.
    #[serde(default)]
    pub registration_scale_set_id: Option<i64>,
    /// Stable overflow delivery identity.
    pub delivery_id: Uuid,
    /// Stable runner identity and unique GitHub registration name suffix.
    pub runner_id: Uuid,
    /// Owning organization registration.
    pub organization_id: Uuid,
    /// Original shared label identity.
    pub label_id: Uuid,
    /// Original label name, retained after deletion.
    pub label_name: String,
    /// Owning scale-set binding.
    pub scale_set_id: Uuid,
    /// Original free-size request.
    pub requested_size: VmSize,
    /// Predicted allocation before the provider fixes its dispatch parameters.
    pub size: VmSize,
    /// Saved image's complete required root allocation.
    pub required_root_disk_bytes: u64,
    /// Exact GitHub runner after a successful registration response.
    pub github_id: Option<i64>,
    /// Trusted reservation time.
    pub created_at: DateTime<Utc>,
}

/// Durable runner identities without a registration secret or callback token.
pub trait OverflowRuntimeStore: Send + Sync {
    /// Save the exact registration response, even before a sandbox runner exists.
    fn save_overflow_registration(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        target: RegistrationTarget,
        github_id: i64,
    ) -> Result<()>;
    /// Read durable overflow supply without provider access or an execution lock.
    fn overflow_runtime_snapshot(&self, workspace_id: Uuid) -> Result<OverflowSnapshot>;
    /// Read all unfinished or retained registration intents in one workspace.
    fn overflow_runner_intents(&self, workspace_id: Uuid) -> Result<Vec<OverflowRunnerIntent>>;
    /// Read saved removal, completed take-out, or absent-owner access facts.
    /// A missing intent has no remaining registration recovery work.
    fn overflow_registration_access_ended(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
    ) -> Result<bool>;
    /// Delete terminal intents whose saved ownership permits metadata cleanup.
    /// Live deliveries, pending take-outs, unknown authority, and history stay.
    fn purge_terminal_overflow_runner_intents(
        &self,
        workspace_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<usize>;
    /// Save a new identity under current organization, label, and provider authority.
    fn create_overflow_runner_intent(
        &self,
        workspace_id: Uuid,
        intent: &OverflowRunnerIntent,
    ) -> Result<()>;
    /// Save the registration identity and update an already materialized runner.
    fn register_overflow_runner(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        github_id: i64,
    ) -> Result<()>;
    /// Remove an intent after the provider delivery and GitHub registration have ended.
    fn remove_overflow_runner_intent(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<()>;
}
