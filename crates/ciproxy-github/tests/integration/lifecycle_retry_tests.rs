//! Public App retries preserve required cleanup and ordered lifecycle transitions.

use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use std::sync::atomic::Ordering;

use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::{Actor, Installation, InstallationAction, InstallationEvent};
use ciproxy_github::error::Error;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::workspaces::Workspace;

use super::app_fixture::{Fixture, installed};
use super::database::Database;
use super::support::{id, setup};

fn created(installation_id: i64) -> InstallationEvent {
    InstallationEvent {
        action: InstallationAction::Created,
        installation: Installation {
            id: installation_id,
            account: installed(installation_id, false).account,
        },
        requester: Some(Actor { id: 41 }),
        sender: Actor { id: 99 },
    }
}

#[tokio::test]
async fn failed_replacement_cleanup_is_replayed_before_activation_in_either_workspace() {
    for explicit_target in [false, true] {
        let database = Database::new();
        let old = setup(&*database.store);
        let fixture = Fixture::new();
        if explicit_target {
            let workspace = Workspace {
                id: id(90),
                name: "new-owner".to_owned(),
                delete_after: None,
            };
            database.store.create_workspace(&workspace).unwrap();
            let mut label = database.store.labels(old.workspace_id).unwrap().remove(0);
            label.id = id(91);
            database.store.create_label(workspace.id, &label).unwrap();
            *fixture.authority.target.lock().unwrap() = Some(workspace.id);
        }
        *fixture.state.current.lock().unwrap() = Some(installed(7, false));
        fixture.output.fail_next.store(true, Ordering::SeqCst);
        let app = fixture.app(database.store.clone());
        assert!(matches!(
            app.installation(created(7)).await,
            Err(Error::AccessUnavailable(503))
        ));
        assert_eq!(
            database
                .store
                .organization(old.workspace_id, old.organization_id)
                .unwrap()
                .unwrap()
                .app_state,
            AppState::Removed
        );
        let retried = app.installation(created(7)).await;
        assert!(
            matches!(&retried, Ok(InstallationOutcome::Organization(organization)) if organization.installation_id == 7)
        );
        let accepted = fixture.output.accepted.lock().unwrap().clone();
        assert_eq!(
            accepted.len(),
            2,
            "explicit_target={explicit_target}: failed cleanup was lost"
        );
        assert!(
            matches!(&accepted[0], LifecycleEvent::Removed { organization, last_organization: true, cause: RemovalCause::Owner }
            if organization.workspace_id == id(1) && organization.id == id(5) && organization.installation_id == 19)
        );
        let expected_workspace = if explicit_target { id(90) } else { id(1) };
        assert!(
            matches!(&accepted[1], LifecycleEvent::Active(organization) if organization.installation_id == 7 && organization.workspace_id == expected_workspace)
        );
        assert_eq!(fixture.output.attempts.lock().unwrap().len(), 3);
    }
}

#[tokio::test]
async fn failed_refresh_suspension_replays_with_matching_state_before_ordered_resume() {
    let database = Database::new();
    let target = setup(&*database.store);
    let fixture = Fixture::new();
    let app = fixture.app(database.store.clone());
    *fixture.state.current.lock().unwrap() = Some(installed(19, true));
    fixture.output.fail_next.store(true, Ordering::SeqCst);
    assert!(matches!(
        app.refresh_repositories(19).await,
        Err(Error::AccessUnavailable(503))
    ));
    assert_eq!(
        database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap()
            .unwrap()
            .app_state,
        AppState::Suspended
    );
    assert!(matches!(app.refresh_repositories(19).await, Ok(())));
    let suspended = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        *fixture.output.accepted.lock().unwrap(),
        vec![LifecycleEvent::Suspended(suspended)]
    );
    *fixture.state.current.lock().unwrap() = Some(installed(19, false));
    fixture.output.fail_next.store(true, Ordering::SeqCst);
    assert!(matches!(
        app.refresh_repositories(19).await,
        Err(Error::AccessUnavailable(503))
    ));
    let active = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    *fixture.state.current.lock().unwrap() = Some(installed(19, true));
    assert!(matches!(app.refresh_repositories(19).await, Ok(())));
    let suspended = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        *fixture.output.accepted.lock().unwrap(),
        vec![
            LifecycleEvent::Suspended(suspended.clone()),
            LifecycleEvent::Active(active),
            LifecycleEvent::Suspended(suspended)
        ]
    );
}

#[tokio::test]
async fn activation_commits_state_and_notification_only_after_preparation_succeeds() {
    let database = Database::new();
    let target = setup(&*database.store);
    database
        .store
        .set_app_suspended(target.workspace_id, target.organization_id, true)
        .unwrap();
    let fixture = Fixture::new();
    fixture.resources.fail_next.store(true, Ordering::SeqCst);
    let app = fixture.app(database.store.clone());
    assert!(matches!(
        app.refresh_repositories(19).await,
        Err(Error::AccessUnavailable(503))
    ));
    assert_eq!(
        database
            .store
            .organization(target.workspace_id, target.organization_id)
            .unwrap()
            .unwrap()
            .app_state,
        AppState::Suspended
    );
    assert_eq!(fixture.output.accepted.lock().unwrap().len(), 0);
    assert!(matches!(app.refresh_repositories(19).await, Ok(())));
    let active = database
        .store
        .organization(target.workspace_id, target.organization_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        *fixture.output.accepted.lock().unwrap(),
        vec![LifecycleEvent::Active(active)]
    );
}
