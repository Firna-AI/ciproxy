//! Workspace deadlines, reinstall recovery, and tenant-scoped deletion.

use chrono::Duration;
use ciproxy_store_interface::jobs::{JobResult, JobState};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use ciproxy_store_interface::settings::OverflowWait;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use uuid::Uuid;

use super::support::installation_claim;
use super::support::{Database, account, at, hash, job, machine, runner};

#[test]
fn workspace_retention_restores_reinstalls_and_purges_only_the_expired_tenant() {
    let database = Database::new();
    let store = &database.store;
    let mut workspace = database.workspace();
    let mut organization = database.organization(&workspace);
    let (claim_id, attempt_digest) = installation_claim::linked(store.as_ref(), &organization);
    let (label, scale_set) = database.label_and_scale_set(&organization);
    let machine = machine();
    store.create_machine(workspace.id, &machine).unwrap();
    let machine_hash = hash();
    store
        .create_scoped_token(
            &Token {
                id: Uuid::new_v4(),
                workspace_id: workspace.id,
                subject: TokenSubject::Machine(machine.id),
                created_at: at(),
                expires_at: None,
                revoked_at: None,
            },
            &machine_hash,
        )
        .unwrap();
    let mut historical_runner = runner(
        &label,
        &scale_set,
        RunnerPlace::Machine {
            machine_id: machine.id,
            first_slot: 0,
        },
    );
    historical_runner.state = RunnerState::Stopped;
    store
        .create_runner(workspace.id, &historical_runner)
        .unwrap();
    let mut historical_job = job(&label, &scale_set);
    historical_job.runner_id = Some(historical_runner.id);
    historical_job.state = JobState::Completed;
    historical_job.result = Some(JobResult::Success);
    historical_job.completed_at = Some(at());
    store.insert_job(workspace.id, &historical_job).unwrap();
    let mut settings = store.settings(workspace.id).unwrap();
    settings.overflow_wait = OverflowWait::Seconds120;
    store.save_settings(workspace.id, &settings).unwrap();
    let account = account();
    store
        .connect_boat(
            workspace.id,
            &account,
            &EncryptedBoatKey::new(vec![1, 2]),
            40_000_000,
        )
        .unwrap();
    settings = store.settings(workspace.id).unwrap();
    store
        .record_app_removal(workspace.id, organization.id, at())
        .unwrap();
    workspace.delete_after = Some(at() + Duration::days(30));
    assert_eq!(
        store.workspace(workspace.id).unwrap(),
        Some(workspace.clone())
    );
    store
        .record_app_removal(workspace.id, organization.id, at() + Duration::days(3))
        .unwrap();
    assert_eq!(
        store.workspace(workspace.id).unwrap(),
        Some(workspace.clone())
    );
    assert!(
        !store
            .purge_expired_workspace(workspace.id, at() + Duration::days(29))
            .unwrap()
    );
    organization.installation_id ^= 1;
    store.install_organization(&organization).unwrap();
    workspace.delete_after = None;
    assert_eq!(
        store.workspace(workspace.id).unwrap(),
        Some(workspace.clone())
    );
    assert_eq!(store.labels(workspace.id).unwrap(), vec![label.clone()]);
    assert_eq!(store.settings(workspace.id).unwrap(), settings);
    assert_eq!(
        store.jobs(workspace.id, organization.id).unwrap(),
        vec![historical_job]
    );
    assert_eq!(
        store.runners(workspace.id, organization.id).unwrap(),
        vec![historical_runner]
    );
    assert_eq!(store.authenticate_token(&machine_hash, at()).unwrap(), None);
    assert_eq!(store.machines(workspace.id).unwrap(), vec![]);
    assert_eq!(store.boat_accounts(workspace.id).unwrap(), vec![]);
    assert!(
        !store
            .purge_expired_workspace(workspace.id, at() + Duration::days(31))
            .unwrap()
    );
    let mut other_workspace = database.workspace();
    let other_organization = database.organization(&other_workspace);
    let (other_label, _) = database.label_and_scale_set(&other_organization);
    store
        .record_app_removal(other_workspace.id, other_organization.id, at())
        .unwrap();
    other_workspace.delete_after = Some(at() + Duration::days(30));
    store
        .record_app_removal(workspace.id, organization.id, at() + Duration::days(40))
        .unwrap();
    workspace.delete_after = Some(at() + Duration::days(70));
    assert_eq!(
        store.workspace(workspace.id).unwrap(),
        Some(workspace.clone())
    );
    assert!(
        !store
            .purge_expired_workspace(workspace.id, at() + Duration::days(69))
            .unwrap()
    );
    assert_eq!(
        store
            .installation_claim(claim_id)
            .unwrap()
            .map(|claim| claim.organization_id),
        Some(Some(organization.id))
    );
    assert!(
        store
            .installation_attempt(&attempt_digest)
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .purge_expired_workspace(workspace.id, at() + Duration::days(70))
            .unwrap()
    );
    assert_eq!(store.workspace(workspace.id).unwrap(), None);
    assert_eq!(
        store
            .installation_claim(claim_id)
            .unwrap()
            .map(|claim| claim.id),
        None
    );
    assert_eq!(
        store
            .installation_attempt(&attempt_digest)
            .unwrap()
            .map(|attempt| attempt.id),
        None
    );
    assert_eq!(
        store.installation_workspace_offers(claim_id, 7).unwrap(),
        Vec::<Uuid>::new()
    );
    assert_eq!(
        store.organization(workspace.id, organization.id).unwrap(),
        None
    );
    assert_eq!(store.labels(workspace.id).unwrap(), vec![]);
    assert_eq!(store.jobs(workspace.id, organization.id).unwrap(), vec![]);
    assert_eq!(
        store.runners(workspace.id, organization.id).unwrap(),
        vec![]
    );
    assert!(
        !store
            .purge_expired_workspace(workspace.id, at() + Duration::days(70))
            .unwrap()
    );
    assert_eq!(
        store.workspace(other_workspace.id).unwrap(),
        Some(other_workspace.clone())
    );
    assert_eq!(store.labels(other_workspace.id).unwrap(), vec![other_label]);
}
