//! Trusted UUID attempts count once, retract unknown runtime, and retain history.

use chrono::Duration;
use ciproxy_control_plane::accounting::Accounting;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::github_timing::JobTimes;
use ciproxy_store_interface::jobs::{Job, JobResult, JobState};
use ciproxy_store_interface::metering::facts::{Completion, JobSource};
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

use super::metering_support::{job, make_runtime, time, totals};
use super::settled::Settled;
use super::support::{Fixture, at};

#[tokio::test]
async fn actual_scale_attempts_count_once_across_replay_restart_reassignment_mapping_and_retention()
{
    let fixture = Fixture::new();
    let mut job = job(&fixture);
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
    let (runtime, accounting) = make_runtime(&fixture);
    runtime
        .settled_job(fixture.workspace, job.clone())
        .await
        .unwrap();
    runtime
        .settled_job(fixture.workspace, job.clone())
        .await
        .unwrap();
    accounting.pass().await;
    assert_eq!(totals(&fixture).0, 1);
    let facts = fixture.store.workspace_usage(fixture.workspace).unwrap();
    assert_eq!(
        facts.jobs[0].started_at,
        time("2026-11-05T12:00:00.000000001Z")
    );
    let first_id = facts.jobs[0].id;
    job.phase = DemandPhase::Completed;
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
    runtime
        .settled_job(fixture.workspace, job.clone())
        .await
        .unwrap();
    accounting.pass().await;
    assert_eq!(totals(&fixture), (1, "0".to_owned()));
    assert_eq!(
        fixture
            .store
            .workspace_usage(fixture.workspace)
            .unwrap()
            .jobs[0]
            .completion,
        Completion::TerminalUnknown
    );
    fixture
        .store
        .save_github_job_times(
            fixture.workspace,
            first_id,
            &JobTimes {
                actions_job_id: 333,
                started_at: job.started_at,
                completed_at: Some(time("2026-11-05T12:01:00.000000001Z")),
                result: None,
                terminal: true,
            },
            at(),
        )
        .unwrap();
    let known = fixture
        .store
        .github_jobs(fixture.workspace, fixture.organization)
        .unwrap()
        .remove(0);
    runtime
        .settled_job(fixture.workspace, known.clone())
        .await
        .unwrap();
    accounting.pass().await;
    assert_eq!(totals(&fixture), (1, "1".to_owned()));
    let (restarted, restarted_accounting) = make_runtime(&fixture);
    restarted
        .settled_job(fixture.workspace, known.clone())
        .await
        .unwrap();
    restarted_accounting.pass().await;
    assert_eq!(totals(&fixture), (1, "1".to_owned()));
    let mut second = fixture.intent.clone();
    second.runner.id = Uuid::new_v4();
    second.runner.github_id = Some(48);
    second.runner.state = RunnerState::Busy;
    second.command_id = Uuid::new_v4();
    second.slot_indices = vec![1];
    fixture
        .store
        .create_own_runner(fixture.workspace, &second)
        .unwrap();
    let mut retry = known;
    retry.assigned_at = Some(time("2026-11-05T12:02:00.000000002Z"));
    retry.started_at = Some(time("2026-11-05T12:02:30.000000001Z"));
    retry.completed_at = Some(time("2026-11-05T12:03:30.000000001Z"));
    retry.runner_github_id = Some(48);
    retry.actions_job_id = None;
    fixture
        .store
        .save_github_job(fixture.workspace, &retry, at())
        .unwrap();
    restarted
        .settled_job(fixture.workspace, retry.clone())
        .await
        .unwrap();
    restarted
        .settled_job(fixture.workspace, retry.clone())
        .await
        .unwrap();
    restarted_accounting.pass().await;
    assert_eq!(totals(&fixture), (2, "2".to_owned()));
    let facts = fixture.store.workspace_usage(fixture.workspace).unwrap();
    assert_eq!(facts.jobs.len(), 2);
    let other_id = facts
        .jobs
        .iter()
        .find(|fact| fact.id != first_id)
        .unwrap()
        .id;
    let numeric = Job {
        id: other_id,
        organization_id: fixture.organization,
        github_id: 333,
        label_id: fixture.intent.runner.label_id,
        scale_set_id: fixture.intent.runner.scale_set_id,
        runner_id: Some(fixture.intent.runner.id),
        name: "same known job".to_owned(),
        workflow: "CI".to_owned(),
        repository: "org/repo".to_owned(),
        git_ref: None,
        state: JobState::Completed,
        result: Some(JobResult::Success),
        queued_at: at() - Duration::minutes(1),
        started_at: Some(time("2026-11-05T12:00:00.000000001Z")),
        completed_at: Some(time("2026-11-05T12:01:00.000000001Z")),
        routing_reason: None,
        cache_source: None,
        machine_handled_cache_counts: None,
    };
    fixture
        .store
        .insert_job(fixture.workspace, &numeric)
        .unwrap();
    restarted
        .settled_job(fixture.workspace, retry.clone())
        .await
        .unwrap();
    restarted_accounting.pass().await;
    assert_eq!(totals(&fixture), (2, "2".to_owned()));
    let facts = fixture.store.workspace_usage(fixture.workspace).unwrap();
    assert_eq!(
        facts
            .accounted_jobs
            .iter()
            .filter(|cursor| cursor.job.source == JobSource::Actions)
            .count(),
        1
    );
    assert_eq!(
        facts
            .accounted_jobs
            .iter()
            .filter(|cursor| cursor.job.source == JobSource::ScaleSet)
            .count(),
        1
    );
    let mut active = retry.clone();
    active.phase = DemandPhase::Started;
    active.assigned_at = Some(at() + Duration::days(90));
    active.started_at = Some(at() + Duration::days(90) + Duration::minutes(1));
    active.completed_at = None;
    fixture
        .store
        .save_github_job(fixture.workspace, &active, at() + Duration::days(90))
        .unwrap();
    fixture
        .store
        .purge_github_jobs(fixture.workspace, at() + Duration::days(91))
        .unwrap();
    let after = fixture.store.workspace_usage(fixture.workspace).unwrap();
    assert_eq!(after.accounted_jobs.len(), 1);
    assert_eq!(after.accounted_jobs[0].job.source, JobSource::Actions);
    restarted_accounting.pass().await;
    assert_eq!(totals(&fixture), (2, "2".to_owned()));
    let retained = fixture
        .store
        .github_jobs(fixture.workspace, fixture.organization)
        .unwrap();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].phase, DemandPhase::Started);
    assert_eq!(retained[0].assigned_at, active.assigned_at);
}
