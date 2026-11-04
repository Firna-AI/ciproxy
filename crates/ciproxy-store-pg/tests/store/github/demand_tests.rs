//! Job demand never regresses on replay or loses a newer pending revision.

use chrono::Duration;
use ciproxy_store_interface::github::{DemandPhase, JobDelivery};
use uuid::Uuid;

use super::super::support::{Database, at};
use super::fixtures::demand;

#[test]
fn uuid_demand_replay_keeps_terminal_state_and_nanosecond_reassignment_order() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let store = &database.store;
    let assigned = demand(&label, &scale_set);
    store
        .save_github_job(workspace.id, &assigned, at())
        .unwrap();
    assert_eq!(
        store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: assigned.clone(),
            revision: 1
        }]
    );
    store
        .confirm_github_job(workspace.id, organization.id, assigned.id, 1)
        .unwrap();
    store
        .save_github_job(workspace.id, &assigned, at())
        .unwrap();
    assert_eq!(
        store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![]
    );
    let mut started = assigned.clone();
    started.phase = DemandPhase::Started;
    started.started_at = Some("2026-11-04T10:46:21.661245605Z".parse().unwrap());
    started.runner_github_id = Some(15);
    store.save_github_job(workspace.id, &started, at()).unwrap();
    let mut completed = started.clone();
    completed.phase = DemandPhase::Completed;
    completed.result = Some("succeeded".to_owned());
    completed.completed_at = Some("2026-11-04T10:46:24.317465059Z".parse().unwrap());
    let mut completion_message = completed.clone();
    completion_message.started_at = None;
    completion_message.runner_github_id = None;
    store
        .save_github_job(workspace.id, &completion_message, at())
        .unwrap();
    store
        .save_github_job(workspace.id, &assigned, at())
        .unwrap();
    store
        .confirm_github_job(workspace.id, organization.id, assigned.id, 1)
        .unwrap();
    assert_eq!(
        store.github_jobs(workspace.id, organization.id).unwrap(),
        vec![completed.clone()]
    );
    assert_eq!(
        store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: completed.clone(),
            revision: 3
        }]
    );
    store
        .confirm_github_job(workspace.id, organization.id, assigned.id, 3)
        .unwrap();
    let mut reassigned = assigned.clone();
    reassigned.assigned_at = Some("2026-11-04T10:46:21.620395767Z".parse().unwrap());
    store
        .save_github_job(workspace.id, &reassigned, at())
        .unwrap();
    store
        .save_github_job(workspace.id, &completed, at())
        .unwrap();
    assert_eq!(
        store
            .pending_github_jobs(workspace.id, organization.id, scale_set.id)
            .unwrap(),
        vec![JobDelivery {
            job: reassigned.clone(),
            revision: 4
        }]
    );
    assert_eq!(
        store.github_jobs(Uuid::new_v4(), organization.id).unwrap(),
        vec![]
    );
}

#[test]
fn completed_demand_retention_deletes_pending_completions_and_preserves_running_jobs() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let store = &database.store;
    let mut completed = demand(&label, &scale_set);
    completed.phase = DemandPhase::Completed;
    completed.completed_at = Some(at() - Duration::days(90));
    let mut pending = completed.clone();
    pending.id = Uuid::from_u128(72);
    let mut running = completed.clone();
    running.id = Uuid::from_u128(73);
    running.phase = DemandPhase::Started;
    running.completed_at = None;
    let mut no_finish = completed.clone();
    no_finish.id = Uuid::from_u128(74);
    no_finish.completed_at = None;
    for job in [&completed, &pending, &running, &no_finish] {
        store
            .save_github_job(workspace.id, job, at() - Duration::days(90))
            .unwrap();
    }
    store
        .save_github_job(workspace.id, &no_finish, at())
        .unwrap();
    store
        .confirm_github_job(workspace.id, organization.id, completed.id, 1)
        .unwrap();
    assert_eq!(
        store
            .purge_github_jobs(workspace.id, at() - Duration::nanoseconds(1))
            .unwrap(),
        0
    );
    assert_eq!(store.purge_github_jobs(workspace.id, at()).unwrap(), 3);
    assert_eq!(
        store.github_jobs(workspace.id, organization.id).unwrap(),
        vec![running]
    );
}
