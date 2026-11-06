//! Trusted job and runner facts inserted through the production store API.

use ciproxy_api::runtime::Runtime;
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::github::{DemandPhase, GithubJob, RepositoryIdentity, RunFacts};
use ciproxy_store_interface::jobs::{Job, JobState};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use uuid::Uuid;

use super::m18_support::{Pages, at};

pub(super) fn runner(pages: &Pages, machine: Option<Uuid>, github: i64) -> Runner {
    let fixture = &pages.fixture;
    let scale = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .remove(0);
    let value = Runner {
        id: Uuid::new_v4(),
        organization_id: fixture.organization.id,
        label_id: scale.label_id,
        scale_set_id: scale.id,
        github_id: Some(github),
        state: RunnerState::Busy,
        size: VmSize {
            vcpu: 2,
            memory_bytes: 8_000_000_000,
            disk_bytes: 58_300_000_001,
        },
        place: machine.map_or_else(
            || RunnerPlace::Boat {
                key_id: None,
                sandbox_id: format!("sandbox-{github}"),
                size_name: "actual-medium".into(),
            },
            |machine_id| RunnerPlace::Machine {
                machine_id,
                first_slot: 2,
            },
        ),
    };
    fixture
        .database
        .store
        .create_runner(fixture.workspace, &value)
        .unwrap();
    value
}

pub(super) fn actions(pages: &Pages, runner: &Runner, github: i64, started: &str) -> Job {
    let value = Job {
        id: Uuid::new_v4(),
        organization_id: runner.organization_id,
        github_id: github,
        label_id: runner.label_id,
        scale_set_id: runner.scale_set_id,
        runner_id: Some(runner.id),
        name: format!("job-{github}"),
        workflow: "CI".into(),
        repository: "customer/private".into(),
        git_ref: Some("refs/heads/main".into()),
        state: JobState::Completed,
        result: Some(ciproxy_store_interface::jobs::JobResult::Success),
        queued_at: at(started) - chrono::Duration::seconds(10),
        started_at: Some(at(started)),
        completed_at: Some(at(started) + chrono::Duration::minutes(1)),
        routing_reason: Some("Own machines were full".into()),
        cache_source: None,
        machine_handled_cache_counts: None,
    };
    pages
        .fixture
        .database
        .store
        .insert_job(pages.fixture.workspace, &value)
        .unwrap();
    value
}

pub(super) fn demand(
    pages: &Pages,
    runner: &Runner,
    actions_job_id: Option<i64>,
    phase: DemandPhase,
    started: Option<&str>,
    completed: Option<&str>,
) -> GithubJob {
    let value = GithubJob {
        id: Uuid::new_v4(),
        organization_id: runner.organization_id,
        scale_set_id: runner.scale_set_id,
        label_id: runner.label_id,
        actions_job_id,
        request_id: 0,
        run_id: 901,
        repository: "customer/private".into(),
        name: "scale-job".into(),
        workflow_ref: "customer/private/.github/workflows/ci.yml@refs/heads/main".into(),
        phase,
        result: (phase == DemandPhase::Completed).then(|| "success".into()),
        assigned_at: Some(at("2026-11-06T11:58:00Z")),
        started_at: started.map(at),
        completed_at: completed.map(at),
        runner_github_id: matches!(phase, DemandPhase::Started | DemandPhase::Completed)
            .then(|| runner.github_id.unwrap()),
        facts: RunFacts {
            id: 901,
            repository: RepositoryIdentity {
                id: 10,
                full_name: "customer/private".into(),
            },
            event: "push".into(),
            head_branch: Some("main".into()),
            head_sha: "trusted-sha".into(),
            head_repository: None,
        },
    };
    pages
        .fixture
        .database
        .store
        .save_github_job(pages.fixture.workspace, &value, pages.fixture.runtime.now())
        .unwrap();
    value
}
