//! API timing preserves listener facts when both writes overlap.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::github::{DemandPhase, JobDelivery};
use ciproxy_store_interface::github_timing::JobTimes;

use crate::store::support::{Database, at};

use super::fixtures::demand;
use super::timing_support::{saved_attempt, saved_completion_observation, timing_after_listener};

#[test]
fn terminal_timing_preserves_listener_completion_during_overlap() {
    preserves_listener_completion(
        true,
        Some("2026-11-04T10:46:24.317465059Z".parse().unwrap()),
    );
}

#[test]
fn running_timing_recovers_actions_id_after_listener_completion_during_overlap() {
    preserves_listener_completion(false, None);
}

fn preserves_listener_completion(terminal: bool, completed_at: Option<DateTime<Utc>>) {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let mut running = demand(&label, &scale_set);
    running.phase = DemandPhase::Started;
    running.runner_github_id = Some(15);
    database
        .store
        .save_github_job(workspace.id, &running, at())
        .unwrap();
    let pending = database
        .store
        .incomplete_github_job_attempts(workspace.id)
        .unwrap();
    assert_eq!(pending.len(), 1);
    let attempt_id = pending[0].attempt_id;
    let mut completion = running.clone();
    completion.phase = DemandPhase::Completed;
    completion.result = Some("succeeded".to_owned());
    completion.runner_github_id = Some(24);
    completion.name = "listener completion".to_owned();
    completion.workflow_ref = "shared/.github/workflows/final.yml@refs/heads/main".to_owned();
    completion.facts.head_sha = "listener-completion-head".to_owned();
    let times = JobTimes {
        actions_job_id: 81,
        started_at: Some("2026-11-04T10:46:21.661245605Z".parse().unwrap()),
        completed_at,
        terminal,
    };
    timing_after_listener(
        &mut database,
        workspace.id,
        attempt_id,
        completion.clone(),
        times.clone(),
    );
    let mut expected = completion;
    expected.actions_job_id = Some(81);
    expected.started_at = Some("2026-11-04T10:46:21.661245605Z".parse().unwrap());
    expected.completed_at = completed_at;
    assert_eq!(
        database
            .store
            .github_jobs(workspace.id, organization.id)
            .unwrap(),
        vec![expected.clone()]
    );
    assert_eq!(
        saved_attempt(&mut database, workspace.id, attempt_id),
        (expected.clone(), Some(at()))
    );
    assert_eq!(
        saved_completion_observation(&mut database, workspace.id, &expected),
        Some(at())
    );
    database
        .store
        .confirm_github_job(workspace.id, organization.id, running.id, 2)
        .unwrap();
    assert_eq!(
        database
            .store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: expected.clone(),
            revision: 3,
        }]
    );
    database
        .store
        .save_github_job_times(workspace.id, attempt_id, &times, at() + Duration::days(1))
        .unwrap();
    assert_eq!(
        saved_attempt(&mut database, workspace.id, attempt_id),
        (expected.clone(), Some(at()))
    );
    assert_eq!(
        saved_completion_observation(&mut database, workspace.id, &expected),
        Some(at())
    );
    assert_eq!(
        database
            .store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: expected,
            revision: 3,
        }]
    );
}
