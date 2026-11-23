//! Read the runner's saved usage for a job and prepare the detail panel.

mod series;

use chrono::DateTime;
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::usage::{JobStep, JobStepTiming, JobUsage};
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::JobState;
use ciproxy_store_interface::vm_size;
use uuid::Uuid;

use crate::error::Result;
use crate::state::{ApiState, query};
use crate::wire::job_usage::{JobUsageLabelMark, JobUsageStep, JobUsageView, JobUsageVmSize};

use self::series::Series;
use super::job_views::seconds;

/// Read usage only for the exact runner of the authorized job.
pub(super) async fn read(
    state: &ApiState,
    workspace: Uuid,
    job: &DashboardJob,
) -> Result<JobUsageView> {
    if job.state == JobState::Running {
        return Ok(JobUsageView::Running);
    }
    let (JobState::Completed, Some(runner), Some(vm)) = (job.state, job.runner_id, job.actual_size)
    else {
        return Ok(JobUsageView::Unavailable);
    };
    let report = query(state.store.clone(), move |store| {
        store.job_usage(workspace, runner)
    })
    .await?;
    Ok(report.map_or(JobUsageView::Unavailable, |report| {
        measured(&report.usage, vm, job.requested_size)
    }))
}

fn measured(usage: &JobUsage, vm: VmSize, requested: Option<vm_size::VmSize>) -> JobUsageView {
    let Some(series) = Series::from_usage(usage) else {
        return JobUsageView::Unavailable;
    };
    let Some(job) = series.whole() else {
        return JobUsageView::Unavailable;
    };
    JobUsageView::Measured {
        job,
        vm: JobUsageVmSize {
            vcpus: vm.vcpu,
            memory_bytes: vm.memory_bytes.to_string(),
        },
        label_mark: label_mark(requested, vm),
        chart: series.chart(),
        steps: steps(&usage.steps, &series),
    }
}

fn label_mark(requested: Option<vm_size::VmSize>, vm: VmSize) -> Option<JobUsageLabelMark> {
    let requested = requested?.bytes().ok()?;
    let vcpus = (requested.vcpu < vm.vcpu).then_some(requested.vcpu);
    let memory_bytes =
        (requested.memory_bytes < vm.memory_bytes).then(|| requested.memory_bytes.to_string());
    (vcpus.is_some() || memory_bytes.is_some()).then_some(JobUsageLabelMark {
        vcpus,
        memory_bytes,
    })
}

fn steps(steps: &[JobStep], series: &Series) -> Vec<JobUsageStep> {
    if steps.is_empty() {
        return Vec::new();
    }
    let start = series.started_at.timestamp_millis();
    let end = series.ended_at.timestamp_millis();
    let first_start = steps
        .iter()
        .find_map(|step| match step.timing {
            JobStepTiming::Ran { started_at_ms, .. } => Some(started_at_ms),
            JobStepTiming::Skipped => None,
        })
        .flatten();
    let last_end = steps
        .iter()
        .rev()
        .find_map(|step| match step.timing {
            JobStepTiming::Ran { ended_at_ms, .. } => Some(ended_at_ms),
            JobStepTiming::Skipped => None,
        })
        .flatten();
    let setup = JobStepTiming::Ran {
        started_at_ms: Some(start),
        ended_at_ms: first_start.map(|at| at.clamp(start, end)),
    };
    let complete = JobStepTiming::Ran {
        started_at_ms: last_end.map(|at| at.clamp(start, end)),
        ended_at_ms: Some(end),
    };
    let mut claimed = vec![false; series.len()];
    [("Set up job", setup)]
        .into_iter()
        .chain(steps.iter().map(|step| (step.name.as_str(), step.timing)))
        .chain([("Complete job", complete)])
        .enumerate()
        .map(|(index, (name, timing))| step(index, name, timing, series, &mut claimed))
        .collect()
}

fn step(
    index: usize,
    name: &str,
    timing: JobStepTiming,
    series: &Series,
    claimed: &mut [bool],
) -> JobUsageStep {
    let (started_at, ended_at) = match timing {
        JobStepTiming::Ran {
            started_at_ms,
            ended_at_ms,
        } => (
            started_at_ms.and_then(DateTime::from_timestamp_millis),
            ended_at_ms.and_then(DateTime::from_timestamp_millis),
        ),
        JobStepTiming::Skipped => (None, None),
    };
    let span = started_at.zip(ended_at).filter(|(start, end)| start <= end);
    JobUsageStep {
        number: (index + 1) as u32,
        name: name.to_owned(),
        script: matches!(name, "Run script" | "Pre Run script" | "Post Run script"),
        skipped: timing == JobStepTiming::Skipped,
        started_at,
        ended_at,
        duration_seconds: span.map(|(start, end)| seconds(start, end)),
        figures: span.and_then(|(start, end)| series.step_figures(start, end, claimed)),
    }
}

#[cfg(test)]
#[path = "_tests_/job_usage_tests.rs"]
mod job_usage_tests;
