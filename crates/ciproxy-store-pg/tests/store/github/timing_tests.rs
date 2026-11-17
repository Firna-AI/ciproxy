//! Timing updates retain terminal observations and exact assignment scope.

use chrono::Duration;
use ciproxy_store_interface::github::{DemandPhase, JobDelivery};
use ciproxy_store_interface::github_timing::JobTimes;

use crate::store::support::{Database, at};

use super::fixtures::demand;
use super::timing_support::{saved_attempt, saved_completion_observation};

#[test]
fn timing_updates_retained_attempt_without_replacing_current_assignment() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let mut running = demand(&label, &scale_set);
    running.phase = DemandPhase::Started;
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
    let mut times = JobTimes {
        actions_job_id: 81,
        started_at: Some("2026-11-04T10:46:21.661245605Z".parse().unwrap()),
        completed_at: None,
        result: None,
        terminal: true,
    };
    database
        .store
        .save_github_job_times(workspace.id, attempt_id, &times, at())
        .unwrap();
    let mut completed = running;
    completed.phase = DemandPhase::Completed;
    completed.actions_job_id = Some(81);
    completed.started_at = Some("2026-11-04T10:46:21.661245605Z".parse().unwrap());
    assert_eq!(
        saved_attempt(&mut database, workspace.id, attempt_id),
        (completed.clone(), Some(at()))
    );
    assert_eq!(
        saved_completion_observation(&mut database, workspace.id, &completed),
        Some(at())
    );
    database
        .store
        .save_github_job_times(workspace.id, attempt_id, &times, at() + Duration::days(1))
        .unwrap();
    assert_eq!(
        saved_attempt(&mut database, workspace.id, attempt_id),
        (completed.clone(), Some(at()))
    );
    assert_eq!(
        saved_completion_observation(&mut database, workspace.id, &completed),
        Some(at())
    );
    assert_eq!(
        database
            .store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: completed.clone(),
            revision: 2,
        }]
    );
    let mut reassigned = demand(&label, &scale_set);
    reassigned.assigned_at = Some("2026-11-04T10:46:21.620395767Z".parse().unwrap());
    database
        .store
        .save_github_job(workspace.id, &reassigned, at())
        .unwrap();
    times.completed_at = Some("2026-11-04T10:46:24.317465059Z".parse().unwrap());
    times.result = Some("failure".to_owned());
    database
        .store
        .save_github_job_times(workspace.id, attempt_id, &times, at() + Duration::days(2))
        .unwrap();
    completed.completed_at = Some("2026-11-04T10:46:24.317465059Z".parse().unwrap());
    completed.result = Some("failure".to_owned());
    assert_eq!(
        saved_attempt(&mut database, workspace.id, attempt_id),
        (completed.clone(), Some(at()))
    );
    assert_eq!(
        database
            .store
            .github_jobs(workspace.id, organization.id)
            .unwrap(),
        vec![reassigned.clone()]
    );
    assert_eq!(
        database
            .store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: reassigned.clone(),
            revision: 3,
        }]
    );
    times.result = Some("success".to_owned());
    times.terminal = false;
    database
        .store
        .save_github_job_times(workspace.id, attempt_id, &times, at() + Duration::days(3))
        .unwrap();
    assert_eq!(
        saved_attempt(&mut database, workspace.id, attempt_id),
        (completed, Some(at()))
    );
    assert_eq!(
        database
            .store
            .github_jobs(workspace.id, organization.id)
            .unwrap(),
        vec![reassigned]
    );
}
