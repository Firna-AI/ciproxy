//! Expected saved authority for results that return after a remote wait.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::gateway::MachineSnapshot;
use crate::github::GithubJob;
use crate::organizations::Organization;
use crate::overflow_delivery::OverflowDeliveryGuard;
use crate::overflow_runtime::OverflowRunnerIntent;
use crate::own_runners::OwnRunner;
use crate::runners::Runner;
use crate::scale_sets::RegistrationTarget;

/// Saved rows that authorized one own-machine remote operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnRunnerGuard {
    /// Exact reservation, command, and execution evidence.
    pub runner: OwnRunner,
    /// Current installation identity and lifecycle, including retained absence.
    pub organization: Option<Organization>,
    /// Current machine report and desired settings, including removal.
    pub machine: Option<MachineSnapshot>,
    /// Current connection fence, including a replaced session.
    pub session_id: Option<Uuid>,
    /// Local admission at dispatch.
    pub admitted: bool,
    /// Saved jobs mapped to this runner at dispatch.
    pub jobs: Vec<GithubJob>,
}

/// A remote result applied only while its saved authority still matches.
pub enum OwnRunnerResult {
    /// Keep the actual response identity for cleanup, and authorize launch if current.
    Registered {
        /// Actual remote target used for the request.
        target: RegistrationTarget,
        /// GitHub identity returned by that request.
        github_id: i64,
    },
    /// Save fresh GitHub busy evidence without replacing newer observations.
    Busy {
        /// GitHub identity from the live Busy answer.
        github_id: i64,
        /// Trusted result observation time.
        at: DateTime<Utc>,
    },
    /// Save a new stop command after GitHub removal or absence.
    Retired {
        /// New durable stop command identity.
        command_id: Uuid,
        /// Trusted stop issue time.
        at: DateTime<Utc>,
        /// Removed GitHub identity, absent after a scoped absence answer.
        github_id: Option<i64>,
    },
    /// Complete an orphan whose GitHub registration ended.
    Stopped {
        /// Last trusted registration identity.
        github_id: Option<i64>,
    },
    /// Confirm current authority before delivering remote preparation facts.
    Prepared,
}

/// Saved overflow identity, lifecycle, and installation at remote dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowRunnerGuard {
    /// Credential-free registration reservation.
    pub intent: OverflowRunnerIntent,
    /// Current installation identity and removal authority.
    pub organization: Option<Organization>,
    /// Current delivery phase and completion evidence.
    pub delivery: Option<OverflowDeliveryGuard>,
    /// Current materialized runner, including trusted Busy state.
    pub runner: Option<Runner>,
    /// Trusted jobs mapped to this registration.
    pub jobs: Vec<GithubJob>,
}

/// Conditional overflow registration result.
pub enum OverflowRunnerResult {
    /// Confirm unchanged saved authority before provider retirement.
    Checked,
    /// Preserve the actual response target and allow startup only under current admission.
    Registered {
        /// Actual remote target used for the request.
        target: RegistrationTarget,
        /// GitHub identity returned by that request.
        github_id: i64,
    },
    /// Save current live GitHub busy proof.
    Busy {
        /// GitHub identity from the live Busy answer.
        github_id: i64,
    },
    /// Forget registration metadata only after GitHub removal and saved withdrawal.
    Retired,
}

/// Exact saved authority after one conditional overflow result write.
pub enum OverflowRunnerWrite {
    /// The result applied and this authority remains saved in the same transaction.
    Applied(Box<OverflowRunnerGuard>),
    /// Retirement applied and removed the registration reservation.
    Retired,
    /// Newer saved authority rejected the result.
    Rejected,
}

/// Scoped overflow authority reads and conditional remote-result writes.
pub trait OverflowResultStore: Send + Sync {
    /// Capture one overflow reservation under row locks.
    fn overflow_runner_guard(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Option<OverflowRunnerGuard>>;
    /// Apply against current lifecycle authority and return its exact saved result.
    fn apply_overflow_runner_result(
        &self,
        workspace_id: Uuid,
        expected: &OverflowRunnerGuard,
        result: OverflowRunnerResult,
    ) -> Result<OverflowRunnerWrite>;
}

/// Scoped own-machine command reads and conditional remote results.
pub trait RuntimeResultStore: Send + Sync {
    /// Read only the command's reservation, without loading a routing snapshot.
    fn own_command(&self, workspace_id: Uuid, command_id: Uuid) -> Result<Option<OwnRunner>>;
    /// Capture current saved authority under row locks in one transaction.
    fn own_runner_guard(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Option<OwnRunnerGuard>>;
    /// Apply the result under row locks; false means newer saved authority won.
    fn apply_own_runner_result(
        &self,
        workspace_id: Uuid,
        expected: &OwnRunnerGuard,
        result: OwnRunnerResult,
    ) -> Result<bool>;
}
