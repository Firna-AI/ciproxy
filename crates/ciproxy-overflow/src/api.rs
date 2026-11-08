//! Public execution inputs and caller responsibilities at the control-plane boundary.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::secret::Secret;
use ciproxy_scaler::actions::StartRunner;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::overflow::record::StartRecord;
use sandbox_interface::SandboxStartRefusal;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::provider::KeyWallets;
use crate::view::OverflowView;

/// Deployment-owned image and callback configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowConfig {
    /// Stable deployment prefix. Workspace identity is added to every mark.
    pub deployment: String,
    /// Shared saved-snapshot name or share code, never an account key.
    pub shared_snapshot: String,
    /// Root geometry for that same image release, shared with scaler snapshots.
    pub root_sizing: RootSizing,
    /// HTTPS control-plane route that accepts the guest's sandbox-only token.
    pub exit_report_url: Secret,
}

/// One authorized Boat start chosen by the scaler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRequest {
    /// Stable delivery identity. Retry this identity until its result is known.
    pub delivery_id: Uuid,
    /// Stable runner identity allocated before GitHub registration.
    pub runner_id: Uuid,
    /// Store organization identity that owns the scale set.
    pub organization_id: Uuid,
    /// Store label identity.
    pub label_id: Uuid,
    /// Store scale-set identity.
    pub scale_set_id: Uuid,
    /// Boat action selected from a consistent scaler snapshot.
    pub action: StartRunner,
    /// One-use GitHub registration. The store receives only its ciphertext.
    pub registration: Secret,
}

/// A reason why accepted queue demand still waits for a runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WaitReason {
    /// No active accepted key or the workspace disabled overflow.
    Disconnected,
    /// The selected size is no longer in the account's eligible catalog.
    SizeUnavailable,
    /// A supplied provider retry time has not arrived.
    RetryAt(DateTime<Utc>),
    /// A billing refusal has not been cleared by a successful account check.
    BillingRequired,
    /// The monthly limit was reached. Running sandboxes continue.
    SpendLimit {
        /// First instant of the next UTC calendar month.
        resumes_at: DateTime<Utc>,
    },
    /// A fresh provider refusal proved that this delivery started no sandbox.
    Refused(SandboxStartRefusal),
}

/// Durable progress returned after executing or resuming one delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartOutcome {
    /// The delivery is saved and needs recover-only reconciliation or startup.
    Pending(StartRecord),
    /// The guest launch was acknowledged. GitHub still owns job assignment.
    Started(StartRecord),
    /// This delivery cannot start now. Demand stays queued in GitHub.
    Waiting(WaitReason),
    /// This delivery has already ended and must never start again.
    Ended(StartRecord),
}

/// Current GitHub and queue facts that prevent deletion of a needed runner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Retirement {
    /// GitHub confirms a job is running. Do not remove the sandbox.
    RunningJob,
    /// Waiting demand still needs the idle registration.
    WaitingDemand,
    /// GitHub removed the idle registration or confirmed its absence.
    Removed,
}

/// The caller owns GitHub registration retirement and current queue checks.
#[unimock(api = RunnerAuthorityMock)]
#[async_trait]
pub trait RunnerAuthority: Send + Sync {
    /// Remove an unneeded idle registration before returning Removed.
    async fn retire(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Retirement>;
}

/// All impure overflow entry points, consumed through dyn by the control plane.
#[async_trait]
pub trait Overflow: Send + Sync {
    /// Verify a key and offer its named wallets without saving the key.
    async fn wallets(&self, key: Secret) -> Result<KeyWallets>;
    /// Grant a pending connection before verification and encryption.
    /// Return ConnectionNotPermitted after last-App removal invalidates that grant.
    async fn connect(
        &self,
        workspace_id: Uuid,
        key_id: Uuid,
        key: Secret,
        wallet: String,
        spend_limit_usd_micros: i64,
    ) -> Result<BoatAccount>;
    /// Stop new starts and retain keys until their last unfinished delivery ends.
    async fn disconnect(&self, workspace_id: Uuid) -> Result<()>;
    /// Refresh account sizes when five minutes old and return current scaler facts.
    async fn view(&self, workspace_id: Uuid) -> Result<OverflowView>;
    /// Persist one scaler action and execute its sandbox delivery.
    async fn start(&self, workspace_id: Uuid, request: StartRequest) -> Result<StartOutcome>;
    /// Resume an existing delivery without creating a second sandbox or runner.
    async fn resume(&self, workspace_id: Uuid, delivery_id: Uuid) -> Result<StartOutcome>;
    /// Accept one report per token and immediately attempt sandbox deletion.
    async fn report_end(&self, token: Secret, runner_id: Uuid) -> Result<()>;
    /// Recover pending deletes and remove marked sandboxes whose runners are unneeded.
    async fn cleanup(&self, workspace_id: Uuid) -> Result<()>;
}
