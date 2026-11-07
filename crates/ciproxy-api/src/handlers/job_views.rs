//! Job wire mappings keep requested resources, actual resources, and timing apart.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::job_views::DashboardJob;
use ciproxy_store_interface::jobs::{CacheSource, JobState};
use rust_decimal::Decimal;

use crate::wire::jobs::{
    JobCacheCounts, JobCacheSource, JobLabel, JobLocation, JobStatus, JobView, JobVmSize,
};

pub(super) fn job(value: DashboardJob, at: DateTime<Utc>) -> JobView {
    let waited_seconds = value
        .started_at
        .zip(value.queued_at)
        .map(|(end, start)| seconds(start, end));
    let end = if value.state == JobState::Running {
        Some(at)
    } else {
        value.completed_at
    };
    let runtime_seconds = value
        .started_at
        .zip(end)
        .map(|(start, end)| seconds(start, end));
    let location = if value.state == JobState::Queued {
        None
    } else if let Some(id) = value.machine_id {
        Some(JobLocation::Machine {
            id: id.to_string(),
            name: value.machine_name,
            first_slot: value.first_slot,
        })
    } else {
        value.boat_size.map(|size| JobLocation::Boat { size })
    };
    let github_url = value.run_id.map(|run| match value.actions_job_id {
        Some(id) => format!(
            "https://github.com/{}/actions/runs/{run}/job/{id}",
            value.repository
        ),
        None => format!("https://github.com/{}/actions/runs/{run}", value.repository),
    });
    JobView {
        id: value.id.to_string(),
        organization_id: value.organization_id.to_string(),
        organization: value.organization,
        name: value.name,
        workflow: value.workflow,
        repository: value.repository,
        git_ref: value.git_ref,
        head_branch: value.head_branch,
        status: match value.state {
            JobState::Queued => JobStatus::Queued,
            JobState::Running => JobStatus::Running,
            JobState::Completed => JobStatus::Completed,
        },
        result: value.result,
        queued_at: value.queued_at,
        started_at: value.started_at,
        completed_at: value.completed_at,
        counts_as_start: value.runner_taken,
        waited_seconds,
        runtime_seconds,
        label: JobLabel {
            name: value.label,
            vcpus: value.requested_size.map(|size| size.vcpus),
            memory_gb: value.requested_size.map(|size| size.memory_gb),
            disk_gb: value.requested_size.map(|size| size.disk_gb),
        },
        location,
        actual_size: value
            .actual_size
            .filter(|_| value.state != JobState::Queued)
            .map(|size| JobVmSize {
                vcpus: size.vcpu,
                memory_bytes: size.memory_bytes.to_string(),
                disk_bytes: size.disk_bytes.to_string(),
            }),
        routing_reason: value.routing_reason,
        github_url,
        cache_source: value.cache_source.map(|source| match source {
            CacheSource::Machine => JobCacheSource::Machine,
            CacheSource::Github => JobCacheSource::Github,
            CacheSource::MachineAndGithub => JobCacheSource::MachineAndGithub,
        }),
        cache_counts: value.cache_counts.map(|counts| JobCacheCounts {
            restored_entries: counts.restored_entries.to_string(),
            restored_bytes: counts.restored_bytes.to_string(),
        }),
    }
}

fn seconds(from: DateTime<Utc>, until: DateTime<Utc>) -> String {
    let duration = until - from;
    let whole = duration.num_seconds();
    let nanos = duration.subsec_nanos();
    (Decimal::from(whole) + Decimal::from(nanos) / Decimal::from(1_000_000_000u64))
        .max(Decimal::ZERO)
        .normalize()
        .to_string()
}
