//! Shared hosted checks for usage reported by own-machine and Boat guests.

use chrono::{DateTime, Utc};
use ciproxy_protocol::usage::{JobStepTiming, JobUsage, UsageValidationError};
use ciproxy_store_interface::error;
use ciproxy_store_interface::job_usage::{JobUsageStore, StoredJobUsage};
use uuid::Uuid;

use crate::job_usage_names::redact_script_name;

const CLOCK_SKEW_MS: i64 = 5 * 60 * 1000;
const MIB_IN_BYTES: u64 = 1_048_576;

/// Saved VM allocation and trusted times for one report.
#[derive(Clone, Copy)]
pub struct UsageLimits {
    /// Actual virtual CPU count, independent of the job label.
    pub vcpus: u32,
    /// Actual VM memory in bytes.
    pub memory_bytes: u64,
    /// First trusted time of the runner's life.
    pub started_at: DateTime<Utc>,
    /// Time the control plane received the report, which bounds this window.
    pub received_at: DateTime<Utc>,
}

impl UsageLimits {
    /// Prepare a report for storage. Bad steps do not discard valid points.
    pub fn prepare(self, mut usage: JobUsage, is_final: bool) -> Option<StoredJobUsage> {
        if let Err(reason) = self.check_points(&usage) {
            tracing::warn!(%reason, "dropped job usage points and steps");
            return None;
        }
        if let Err(reason) = self.check_steps(&usage) {
            tracing::warn!(%reason, "dropped job usage steps");
            usage.steps.clear();
        }
        for step in &mut usage.steps {
            redact_script_name(&mut step.name);
        }
        Some(StoredJobUsage {
            usage,
            is_final,
            received_at: self.received_at,
        })
    }

    /// Write final usage when the points pass the checks.
    pub fn save_final(
        self,
        store: &dyn JobUsageStore,
        workspace_id: Uuid,
        runner_id: Uuid,
        usage: JobUsage,
    ) -> error::Result<()> {
        if let Some(report) = self.prepare(usage, true) {
            store.save_job_usage(workspace_id, runner_id, &report)?;
        }
        Ok(())
    }

    fn check_points(self, usage: &JobUsage) -> Result<(), UsageError> {
        usage.validate_points()?;
        let cpu_limit = u64::from(self.vcpus) * 1000;
        if usage
            .cpu_milli_vcpu
            .iter()
            .any(|&cpu| u64::from(cpu) > cpu_limit)
        {
            return Err(UsageError::CpuLimit);
        }
        if usage
            .peak_memory_mib
            .iter()
            .any(|&memory| u64::from(memory) * MIB_IN_BYTES > self.memory_bytes)
        {
            return Err(UsageError::MemoryLimit);
        }
        let start = usage.first_point_at_ms;
        let span = usage.cpu_milli_vcpu.len() as i64 * i64::from(usage.point_seconds) * 1000;
        if !self.contains(start)
            || !start
                .checked_add(span)
                .is_some_and(|end| self.contains(end))
        {
            return Err(UsageError::OutsideRunnerLife);
        }
        Ok(())
    }

    fn check_steps(self, usage: &JobUsage) -> Result<(), UsageError> {
        usage.validate_steps()?;
        for step in &usage.steps {
            if step.name.contains('\0') {
                return Err(UsageError::StepNameNul);
            }
            if let JobStepTiming::Ran {
                started_at_ms,
                ended_at_ms,
            } = step.timing
                && [started_at_ms, ended_at_ms]
                    .into_iter()
                    .flatten()
                    .any(|at| !self.contains(at))
            {
                return Err(UsageError::OutsideRunnerLife);
            }
        }
        Ok(())
    }

    fn contains(self, at_ms: i64) -> bool {
        let start = self.started_at.timestamp_millis() - CLOCK_SKEW_MS;
        let end = self.received_at.timestamp_millis() + CLOCK_SKEW_MS;
        (start..=end).contains(&at_ms)
    }
}

#[derive(Debug, thiserror::Error)]
enum UsageError {
    #[error("[overflow/usage] {0}")]
    Structure(#[from] UsageValidationError),
    #[error("[overflow/usage] CPU exceeds VM allocation")]
    CpuLimit,
    #[error("[overflow/usage] memory exceeds VM allocation")]
    MemoryLimit,
    #[error("[overflow/usage] time is outside the runner's life")]
    OutsideRunnerLife,
    #[error("[overflow/usage] step name contains a null character")]
    StepNameNul,
}

#[cfg(test)]
#[path = "_tests_/job_usage_tests.rs"]
mod job_usage_tests;
