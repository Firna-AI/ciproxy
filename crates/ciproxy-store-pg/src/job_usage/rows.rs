//! Mappings between job usage reports and PostgreSQL rows.

use chrono::{DateTime, Utc};
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_schema::job_usage::{job_usage_samples, job_usage_steps};
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::job_usage::StoredJobUsage;
use diesel::prelude::*;
use uuid::Uuid;

use crate::platform_text::{parse_platform, platform_text};

/// One complete series for a runner.
#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = job_usage_samples)]
pub(super) struct SampleRow {
    workspace_id: Uuid,
    runner_id: Uuid,
    platform: String,
    first_point_at_ms: i64,
    point_seconds: i64,
    cpu_milli_vcpu: Vec<i64>,
    peak_memory_mib: Vec<i64>,
    oom_kills: Option<Vec<i64>>,
    is_final: bool,
    received_at: DateTime<Utc>,
}

impl SampleRow {
    /// Preserve the units and full range of the protocol series.
    pub(super) fn from_report(
        workspace_id: Uuid,
        runner_id: Uuid,
        report: &StoredJobUsage,
    ) -> Self {
        let usage = &report.usage;
        Self {
            workspace_id,
            runner_id,
            platform: platform_text(usage.platform).to_owned(),
            first_point_at_ms: usage.first_point_at_ms,
            point_seconds: i64::from(usage.point_seconds),
            cpu_milli_vcpu: widen_points(&usage.cpu_milli_vcpu),
            peak_memory_mib: widen_points(&usage.peak_memory_mib),
            oom_kills: usage.oom_kills.as_deref().map(widen_points),
            is_final: report.is_final,
            received_at: report.received_at,
        }
    }

    /// Rebuild the report with the steps from the same snapshot.
    pub(super) fn into_report(self, steps: Vec<JobStep>) -> Result<StoredJobUsage> {
        Ok(StoredJobUsage {
            usage: JobUsage {
                platform: parse_platform(&self.platform)?,
                first_point_at_ms: self.first_point_at_ms,
                point_seconds: u32::try_from(self.point_seconds).internal()?,
                cpu_milli_vcpu: narrow_points(self.cpu_milli_vcpu)?,
                peak_memory_mib: narrow_points(self.peak_memory_mib)?,
                oom_kills: self.oom_kills.map(narrow_points).transpose()?,
                steps,
            },
            is_final: self.is_final,
            received_at: self.received_at,
        })
    }
}

fn widen_points(points: &[u32]) -> Vec<i64> {
    points.iter().copied().map(i64::from).collect()
}

fn narrow_points(points: Vec<i64>) -> Result<Vec<u32>> {
    points
        .into_iter()
        .map(|point| u32::try_from(point).internal())
        .collect()
}

/// One step at its position in the report.
#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = job_usage_steps)]
pub(super) struct StepRow {
    workspace_id: Uuid,
    runner_id: Uuid,
    position: i32,
    name: String,
    started_at_ms: Option<i64>,
    ended_at_ms: Option<i64>,
    skipped: bool,
}

impl StepRow {
    /// Keep the supplied name and distinguish a skip from missing times.
    pub(super) fn from_step(
        workspace_id: Uuid,
        runner_id: Uuid,
        position: usize,
        step: &JobStep,
    ) -> Result<Self> {
        let (started_at_ms, ended_at_ms, skipped) = match step.timing {
            JobStepTiming::Ran {
                started_at_ms,
                ended_at_ms,
            } => (started_at_ms, ended_at_ms, false),
            JobStepTiming::Skipped => (None, None, true),
        };
        Ok(Self {
            workspace_id,
            runner_id,
            position: i32::try_from(position).internal()?,
            name: step.name.clone(),
            started_at_ms,
            ended_at_ms,
            skipped,
        })
    }

    /// Read the stored name and timing.
    pub(super) fn into_step(self) -> JobStep {
        let timing = if self.skipped {
            JobStepTiming::Skipped
        } else {
            JobStepTiming::Ran {
                started_at_ms: self.started_at_ms,
                ended_at_ms: self.ended_at_ms,
            }
        };
        JobStep {
            name: self.name,
            timing,
        }
    }
}
