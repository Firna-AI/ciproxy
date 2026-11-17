//! One trusted join from scale-set demand through a registered runner to its Worker.

use ciproxy_protocol::identifiers::JobId;
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::own_runners::{OwnRunner, RoutingSnapshot};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use uuid::Uuid;

/// Saved scale-set facts and the exact runner that GitHub assigned to them.
pub(crate) struct Assignment<'a> {
    /// Trusted scale-set record, whose ID stays in the hosted namespace.
    pub(crate) job: &'a GithubJob,
    /// Exact saved runner and its independently reported Worker job identity.
    pub(crate) runner: &'a OwnRunner,
}

impl<'a> Assignment<'a> {
    /// Resolve only an unambiguous registered identity within its organization and pool.
    pub(crate) fn resolve(input: &'a RoutingSnapshot, job: &'a GithubJob) -> Option<Self> {
        if job.phase < DemandPhase::Started {
            return None;
        }
        let github_id = job.runner_github_id?;
        let mut runners = input.runners.iter().filter(|runner| {
            runner.runner.github_id == Some(github_id)
                && runner.runner.organization_id == job.organization_id
                && runner.runner.scale_set_id == job.scale_set_id
        });
        let runner = runners.next()?;
        if runners.next().is_some() {
            tracing::warn!(
                organization_id=%job.organization_id,
                scale_set_id=%job.scale_set_id,
                "multiple runner registrations match a saved assignment"
            );
            return None;
        }
        Some(Self { job, runner })
    }

    /// Authorize a Worker identity only through its calling machine's saved binding.
    pub(crate) fn for_worker(
        input: &'a RoutingSnapshot,
        machine_id: Uuid,
        worker_job_id: &JobId,
    ) -> Option<Self> {
        let mut assignments = input.jobs.iter().filter_map(|job| {
            let assignment = Self::resolve(input, job)?;
            let runner = assignment.runner;
            (job.phase == DemandPhase::Started
                && runner.worker_job_id.as_ref().is_some_and(|id| &id.0 == worker_job_id)
                && matches!(runner.runner.state, RunnerState::Busy | RunnerState::Stopping)
                && matches!(runner.runner.place, RunnerPlace::Machine { machine_id: owner, .. } if owner == machine_id))
            .then_some(assignment)
        });
        let assignment = assignments.next()?;
        if assignments.next().is_some() {
            tracing::warn!(%machine_id, "multiple started assignments match a machine job");
            return None;
        }
        Some(assignment)
    }
}
