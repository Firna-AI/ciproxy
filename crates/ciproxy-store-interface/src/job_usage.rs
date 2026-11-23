//! Storage for whole-VM samples and steps by runner identity.

use chrono::{DateTime, Utc};
use ciproxy_protocol::usage::JobUsage;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;

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

/// Reads and writes the reports of runners in one workspace.
#[unimock(api = JobUsageStoreMock)]
pub trait JobUsageStore: Send + Sync {
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
