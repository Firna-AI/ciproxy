//! Reports retain their sample units, step order, and finality.

use chrono::Duration;
use ciproxy_protocol::usage::{JobStep, JobStepTiming};
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::jobs::JobState;

use super::support::job_usage::{assert_job_usage_deleted, boat_runner, sample_report};
use super::support::{Database, at, job};

#[test]
fn job_usage_round_trips_samples_and_steps_for_the_dashboard_runner() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let runner = boat_runner(&database, workspace.id, &label, &scale_set);
    let store = &database.store;
    assert_eq!(store.job_usage(workspace.id, runner.id).unwrap(), None);

    let mut report = sample_report();
    report.usage.steps.extend([
        JobStep {
            name: "Compile".to_owned(),
            timing: JobStepTiming::Ran {
                started_at_ms: None,
                ended_at_ms: None,
            },
        },
        JobStep {
            name: "Optional step".to_owned(),
            timing: JobStepTiming::Skipped,
        },
        JobStep {
            name: "Upload".to_owned(),
            timing: JobStepTiming::Ran {
                started_at_ms: Some(1_793_793_602_500),
                ended_at_ms: None,
            },
        },
    ]);
    assert!(
        store
            .save_job_usage(workspace.id, runner.id, &report)
            .unwrap()
    );

    let mut actions_job = job(&label, &scale_set);
    actions_job.runner_id = Some(runner.id);
    actions_job.state = JobState::Running;
    database
        .store
        .insert_job(workspace.id, &actions_job)
        .unwrap();
    let jobs = database.store.dashboard_jobs(workspace.id, at()).unwrap();
    assert_eq!(jobs.len(), 1);
    let dashboard_runner_id = jobs[0].runner_id.unwrap();
    assert_eq!(
        store.job_usage(workspace.id, dashboard_runner_id).unwrap(),
        Some(report.clone())
    );

    let other = database.workspace();
    assert_eq!(store.job_usage(other.id, runner.id).unwrap(), None);
    assert!(matches!(
        store.save_job_usage(other.id, runner.id, &report),
        Err(Error::InvalidReference)
    ));
}

#[test]
fn job_usage_keeps_the_newest_partial_until_a_final_report_replaces_it() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let runner = boat_runner(&database, workspace.id, &label, &scale_set);
    let store = &database.store;

    let older = sample_report();
    assert!(
        store
            .save_job_usage(workspace.id, runner.id, &older)
            .unwrap()
    );
    let mut newer = sample_report();
    newer.received_at = at() + Duration::seconds(30);
    newer.usage.point_seconds = 4;
    newer.usage.cpu_milli_vcpu = vec![1_500];
    newer.usage.peak_memory_mib = vec![4_096];
    newer.usage.oom_kills = Some(vec![2]);
    newer.usage.steps.clear();
    assert!(
        store
            .save_job_usage(workspace.id, runner.id, &newer)
            .unwrap()
    );
    assert_eq!(
        store.job_usage(workspace.id, runner.id).unwrap(),
        Some(newer.clone())
    );
    assert!(
        !store
            .save_job_usage(workspace.id, runner.id, &older)
            .unwrap()
    );
    assert_eq!(
        store.job_usage(workspace.id, runner.id).unwrap(),
        Some(newer)
    );

    let mut final_report = sample_report();
    final_report.is_final = true;
    final_report.received_at = at() + Duration::seconds(29);
    assert!(
        store
            .save_job_usage(workspace.id, runner.id, &final_report)
            .unwrap()
    );

    let mut late_partial = older;
    late_partial.received_at = at() + Duration::seconds(31);
    late_partial.usage.steps.clear();
    late_partial.usage.cpu_milli_vcpu = vec![0, 0];
    assert!(
        !store
            .save_job_usage(workspace.id, runner.id, &late_partial)
            .unwrap()
    );
    assert_eq!(
        store.job_usage(workspace.id, runner.id).unwrap(),
        Some(final_report)
    );
}

#[test]
fn finished_actions_job_purge_removes_job_usage_at_ninety_days() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let runner = boat_runner(&database, workspace.id, &label, &scale_set);
    let mut completed = job(&label, &scale_set);
    completed.runner_id = Some(runner.id);
    completed.state = JobState::Completed;
    completed.completed_at = Some(at());
    database.store.insert_job(workspace.id, &completed).unwrap();
    let report = sample_report();
    assert!(
        database
            .store
            .save_job_usage(workspace.id, runner.id, &report)
            .unwrap()
    );

    let expiry = at() + Duration::days(90);
    assert_eq!(
        database
            .store
            .purge_finished_jobs(workspace.id, expiry - Duration::seconds(1))
            .unwrap(),
        0
    );
    assert_eq!(
        database.store.job_usage(workspace.id, runner.id).unwrap(),
        Some(report)
    );
    assert_eq!(
        database
            .store
            .purge_finished_jobs(workspace.id, expiry)
            .unwrap(),
        1
    );
    assert_job_usage_deleted(&mut database, workspace.id, runner.id);
}

#[test]
fn app_removal_purges_job_usage_only_for_the_removed_organization() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let removed = database.organization(&workspace);
    let active = database.organization(&workspace);
    let report = sample_report();

    let runner_ids = [&removed, &active].map(|organization| {
        let (label, scale_set) = database.label_and_scale_set(organization);
        let runner = boat_runner(&database, workspace.id, &label, &scale_set);
        let mut record = job(&label, &scale_set);
        record.runner_id = Some(runner.id);
        database.store.insert_job(workspace.id, &record).unwrap();
        assert!(
            database
                .store
                .save_job_usage(workspace.id, runner.id, &report)
                .unwrap()
        );
        runner.id
    });
    let [removed_runner, active_runner] = runner_ids;

    database
        .store
        .record_app_removal(workspace.id, removed.id, at())
        .unwrap();
    let expiry = at() + Duration::days(30);
    assert_eq!(
        database
            .store
            .purge_removed_organizations(workspace.id, expiry - Duration::seconds(1))
            .unwrap(),
        0
    );
    assert_eq!(
        database
            .store
            .job_usage(workspace.id, removed_runner)
            .unwrap(),
        Some(report.clone())
    );
    assert_eq!(
        database
            .store
            .purge_removed_organizations(workspace.id, expiry)
            .unwrap(),
        1
    );
    assert_job_usage_deleted(&mut database, workspace.id, removed_runner);
    assert_eq!(
        database
            .store
            .job_usage(workspace.id, active_runner)
            .unwrap(),
        Some(report)
    );
}
