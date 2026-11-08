//! Credential-free GitHub registration intents for recoverable Boat startup.

use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::overflow::store::OverflowSnapshot;
use crate::scale_sets::RegistrationTarget;

/// Stable identities saved before a one-use registration is requested.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BoatRunnerIntent {
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
pub trait BoatRuntimeStore: Send + Sync {
    /// Save the exact registration response, even before a sandbox runner exists.
    fn save_boat_registration(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        target: RegistrationTarget,
        github_id: i64,
    ) -> Result<()>;
    /// Read durable Boat supply without provider access or the overflow execution lock.
    fn boat_runtime_snapshot(&self, workspace_id: Uuid) -> Result<OverflowSnapshot>;
    /// Read all unfinished or retained registration intents in one workspace.
    fn boat_runner_intents(&self, workspace_id: Uuid) -> Result<Vec<BoatRunnerIntent>>;
    /// Read saved removal, completed take-out, or absent-owner access facts.
    /// A missing intent has no remaining registration recovery work.
    fn boat_registration_access_ended(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<bool>;
    /// Delete terminal intents whose saved ownership permits metadata cleanup.
    /// Live deliveries, pending take-outs, unknown authority, and history stay.
    fn purge_terminal_boat_runner_intents(
        &self,
        workspace_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<usize>;
    /// Save a new identity only under current organization, label, and Boat authority.
    fn create_boat_runner_intent(
        &self,
        workspace_id: Uuid,
        intent: &BoatRunnerIntent,
    ) -> Result<()>;
    /// Save the registration identity and update an already materialized runner.
    fn register_boat_runner(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        github_id: i64,
    ) -> Result<()>;
    /// Remove an intent after the provider delivery and GitHub registration have ended.
    fn remove_boat_runner_intent(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<()>;
}
