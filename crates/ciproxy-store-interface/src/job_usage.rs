//! Storage for whole-VM samples and steps by runner identity.

use chrono::{DateTime, Utc};
use ciproxy_protocol::usage::JobUsage;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::overflow::record::StartRecord;
use crate::secrets::TokenHash;

/// The saved report for one job runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredJobUsage {
    /// The samples and ordered steps, with names safe to store.
    pub usage: JobUsage,
    /// True only for usage from an accepted runner exit report.
    pub is_final: bool,
    /// When the control plane received this report.
    pub received_at: DateTime<Utc>,
}

/// Saved allocation and first time evidence for a runner on its own machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnUsageTarget {
    /// Actual virtual CPU count.
    pub vcpus: u32,
    /// Actual memory in bytes.
    pub memory_bytes: u64,
    /// Earliest available command or execution evidence.
    pub started_at: DateTime<Utc>,
}

/// Delivery authenticated by its sandbox token and runner identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatUsageTarget {
    /// Workspace derived from the callback token.
    pub workspace_id: Uuid,
    /// Saved delivery, including its actual provider size and lifecycle.
    pub record: StartRecord,
}

/// Result of a conditional non-final write for a Boat sandbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoatSampleWrite {
    /// The report replaced the previous series, if there was one.
    Saved,
    /// The delivery no longer accepts samples, or a final series exists.
    Closed,
    /// The previous sample is newer than the permitted receipt time.
    TooSoon,
}

/// Reads and writes the reports of runners in one workspace.
#[unimock(api = JobUsageStoreMock)]
pub trait JobUsageStore: Send + Sync {
    /// Read an own runner's usage limits only for the machine that owns it.
    fn own_usage_target(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Option<OwnUsageTarget>>;

    /// Read a Boat runner's delivery only for its exact sandbox token.
    fn boat_usage_target(
        &self,
        hash: &TokenHash,
        runner_id: Uuid,
    ) -> Result<Option<BoatUsageTarget>>;

    /// Save a validated non-final report only before the delivery end or exit claim.
    /// Check the preceding receipt time in the same transaction as the write.
    fn save_boat_sample(
        &self,
        workspace_id: Uuid,
        delivery_id: Uuid,
        runner_id: Uuid,
        report: &StoredJobUsage,
        previous_at_or_before: DateTime<Utc>,
    ) -> Result<BoatSampleWrite>;

    /// Save validated samples and steps as one report. The caller sanitizes names.
    ///
    /// Returns false if a final report or a report of the same kind with an equal
    /// or later arrival time already takes precedence. Final reports take
    /// precedence over partial reports regardless of arrival time.
    fn save_job_usage(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        report: &StoredJobUsage,
    ) -> Result<bool>;

    /// Read the report for the exact runner, with steps in runner order.
    fn job_usage(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<Option<StoredJobUsage>>;
}
