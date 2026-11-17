//! Expected saved authority for results that return after a remote wait.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::boat_runtime::BoatRunnerIntent;
use crate::error::Result;
use crate::gateway::MachineSnapshot;
use crate::github::GithubJob;
use crate::organizations::Organization;
use crate::overflow::record::StartRecord;
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

/// Saved Boat identity, lifecycle, and installation at remote dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatRunnerGuard {
    /// Credential-free registration reservation.
    pub intent: BoatRunnerIntent,
    /// Current installation identity and removal authority.
    pub organization: Option<Organization>,
    /// Current delivery phase and completion evidence.
    pub delivery: Option<StartRecord>,
    /// Current materialized runner, including trusted Busy state.
    pub runner: Option<Runner>,
    /// Trusted jobs mapped to this registration.
    pub jobs: Vec<GithubJob>,
}

/// Conditional Boat registration result.
pub enum BoatRunnerResult {
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

/// Scoped Boat authority reads and atomic conditional remote-result writes.
pub trait BoatResultStore: Send + Sync {
    /// Capture one Boat reservation under row locks.
    fn boat_runner_guard(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Option<BoatRunnerGuard>>;
    /// Apply only against the saved Boat reservation, delivery, and installation.
    fn apply_boat_runner_result(
        &self,
        workspace_id: Uuid,
        expected: &BoatRunnerGuard,
        result: BoatRunnerResult,
    ) -> Result<bool>;
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
