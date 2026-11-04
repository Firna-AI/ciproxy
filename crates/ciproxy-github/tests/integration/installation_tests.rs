//! Current GitHub state defeats delayed events without losing requester ownership.

use std::sync::atomic::Ordering;

use chrono::Duration;
use ciproxy_github::app::webhook::{Actor, Installation, InstallationAction, InstallationEvent};
use ciproxy_github::app::{InstallationOutcome, LifecycleEvent};
use ciproxy_github::error::Error;
use ciproxy_store_interface::organizations::AppState;

use super::app_fixture::{Fixture, at, installed};
use super::database::Database;
use super::support::setup;

fn event(action: InstallationAction, installation_id: i64) -> InstallationEvent {
    InstallationEvent {
        action,
        installation: Installation {
            id: installation_id,
            account: installed(installation_id, false).account,
        },
        requester: Some(Actor { id: 41 }),
        sender: Actor { id: 99 },
    }
}

#[tokio::test]
async fn authoritative_installation_ignores_old_creation_and_retries_absent_state_without_mutation()
{
    let database = Database::new();
    let target = setup(&*database.store);
    let fixture = Fixture::new();
    let app = fixture.app(database.store.clone());
    app.installation(event(InstallationAction::Deleted, 19))
        .await
        .unwrap();
    assert_eq!(fixture.state.reads.load(Ordering::SeqCst), 0);
    *fixture.state.current.lock().unwrap() = Some(installed(7, false));
    let outcome = app
        .installation(event(InstallationAction::Created, 7))
        .await
        .unwrap();
    let mut expected = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    assert_eq!(expected.installation_id, 7);
    assert_eq!(outcome, InstallationOutcome::Organization(expected.clone()));
    assert_eq!(
        app.installation(event(InstallationAction::Created, 19))
            .await
            .unwrap(),
        InstallationOutcome::Ignored
    );
    assert_eq!(
        database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap(),
        Some(expected.clone())
    );
    assert_eq!(*fixture.authority.requesters.lock().unwrap(), vec![41]);
    let reads = fixture.state.reads.load(Ordering::SeqCst);
    *fixture.state.current.lock().unwrap() = None;
    app.installation(event(InstallationAction::Deleted, 7))
        .await
        .unwrap();
    assert_eq!(fixture.state.reads.load(Ordering::SeqCst), reads);
    expected = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    assert_eq!(expected.app_state, AppState::Removed);
    assert!(matches!(
        app.installation(event(InstallationAction::Created, 19))
            .await,
        Err(Error::InstallationPending)
    ));
    assert_eq!(
        database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap(),
        Some(expected)
    );
    assert!(
        database
            .store
            .purge_expired_workspace(target.workspace_id, at() + Duration::days(30))
            .unwrap()
    );
    assert!(matches!(
        app.installation(event(InstallationAction::Created, 19))
            .await,
        Err(Error::InstallationPending)
    ));
    assert_eq!(
        database.store.organizations_by_github_id(17).unwrap(),
        vec![]
    );
    assert_eq!(database.store.workspace(target.workspace_id).unwrap(), None);
    assert_eq!(*fixture.authority.requesters.lock().unwrap(), vec![41]);
}

#[tokio::test]
async fn delayed_suspend_follows_current_active_installation_after_unsuspend() {
    let database = Database::new();
    let target = setup(&*database.store);
    database
        .store
        .set_app_suspended(target.workspace_id, target.organization_id, true)
        .unwrap();
    let fixture = Fixture::new();
    let app = fixture.app(database.store.clone());
    app.installation(event(InstallationAction::Unsuspend, 19))
        .await
        .unwrap();
    app.installation(event(InstallationAction::Suspend, 19))
        .await
        .unwrap();
    let organization = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    assert_eq!(organization.app_state, AppState::Active);
    assert_eq!(
        *fixture.output.0.lock().unwrap(),
        vec![
            LifecycleEvent::Active(organization.clone()),
            LifecycleEvent::Active(organization)
        ]
    );
}

#[tokio::test]
async fn replaced_remote_bindings_and_recreated_registration_keep_organization_group_identity() {
    let database = Database::new();
    let target = setup(&*database.store);
    let fixture = Fixture::new();
    let app = fixture.app(database.store.clone());
    fixture.remote.group_id.store(97, Ordering::SeqCst);
    fixture.resources.base_id.store(83, Ordering::SeqCst);
    let _lease = database
        .store
        .try_lock_scale_set(
            target.workspace_id,
            target.organization_id,
            target.scale_set_id,
        )
        .unwrap()
        .unwrap();
    app.refresh_repositories(19).await.unwrap();
    let saved = database
        .store
        .scale_sets(target.workspace_id, target.organization_id)
        .unwrap();
    assert_eq!(saved[0].id, target.scale_set_id);
    assert_eq!((saved[0].github_id, saved[0].runner_group_id), (83, 97));
    assert!(
        database
            .store
            .try_lock_scale_set(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id
            )
            .unwrap()
            .is_none()
    );
    assert_eq!(
        *fixture.resources.ensures.lock().unwrap(),
        vec![("ciproxy".to_owned(), 97, Some(53))]
    );
    assert_eq!(
        fixture.remote.policies.lock().unwrap()[0].saved_id,
        Some(47)
    );
    assert_eq!(
        fixture.remote.policies.lock().unwrap()[0].name,
        "ciproxy-17"
    );
    app.installation(event(InstallationAction::Deleted, 19))
        .await
        .unwrap();
    assert!(
        database
            .store
            .purge_expired_workspace(target.workspace_id, at() + Duration::days(30))
            .unwrap()
    );
    *fixture.state.current.lock().unwrap() = Some(installed(7, false));
    app.installation(event(InstallationAction::Created, 7))
        .await
        .unwrap();
    let recreated = database.store.organizations_by_github_id(17).unwrap();
    assert_eq!(recreated.len(), 1);
    assert_ne!(recreated[0].id, target.organization_id);
    let policies = fixture.remote.policies.lock().unwrap();
    assert_eq!(policies.last().unwrap().name, "ciproxy-17");
    assert_eq!(policies.last().unwrap().saved_id, None);
    assert_eq!(*fixture.authority.admins.lock().unwrap(), vec![41]);
}

#[tokio::test]
async fn suspended_installation_retry_delivers_pending_initial_admin_without_activation() {
    let database = Database::new();
    let fixture = Fixture::new();
    *fixture.state.current.lock().unwrap() = Some(installed(19, true));
    fixture
        .authority
        .fail_next_admin
        .store(true, Ordering::SeqCst);
    let app = fixture.app(database.store.clone());
    assert!(matches!(
        app.installation(event(InstallationAction::Created, 19))
            .await,
        Err(Error::AccessUnavailable(503))
    ));
    let saved = database
        .store
        .organizations_by_github_id(17)
        .unwrap()
        .remove(0);
    assert_eq!(saved.app_state, AppState::Suspended);
    assert_eq!(
        database
            .store
            .pending_initial_admin(saved.workspace_id)
            .unwrap(),
        Some(41)
    );
    assert_eq!(fixture.output.0.lock().unwrap().len(), 0);
    let result = app
        .installation(event(InstallationAction::Created, 19))
        .await;
    assert!(matches!(result, Ok(InstallationOutcome::Organization(_))));
    assert_eq!(*fixture.authority.admins.lock().unwrap(), vec![41, 41]);
    assert_eq!(
        database
            .store
            .pending_initial_admin(saved.workspace_id)
            .unwrap(),
        None
    );
    assert_eq!(
        database.store.organizations_by_github_id(17).unwrap(),
        vec![saved.clone()]
    );
    assert_eq!(
        *fixture.output.0.lock().unwrap(),
        vec![LifecycleEvent::Suspended(saved)]
    );
    assert_eq!(fixture.remote.policies.lock().unwrap().len(), 0);
}
