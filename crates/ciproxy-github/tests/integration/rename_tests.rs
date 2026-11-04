//! Verified organization renames reach durable state and subsequent SDK routes.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use ciproxy_github::app::LifecycleEvent;
use ciproxy_github::app::webhook::{Actor, Installation, InstallationAction, InstallationEvent};
use ciproxy_github::error;
use ciproxy_github::installation_state::AppInstallationClient;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::persistence::session::{ListenerTarget, SessionStore};
use ciproxy_github::scale_sets::ScaleSetFactory;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::github::GithubInstallationState;
use ciproxy_store_interface::organizations::AppState;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};

use super::app_fixture::{Fixture, installed};
use super::database::Database;
use super::fake_http::Server;
use super::support::{Factory, FixedJwt, FixedRuntime, id};

fn event(action: InstallationAction) -> InstallationEvent {
    InstallationEvent {
        action,
        installation: Installation {
            id: 19,
            account: installed(19, false).account,
        },
        requester: Some(Actor { id: 41 }),
        sender: Actor { id: 99 },
    }
}

#[tokio::test]
async fn verified_rename_persists_active_and_suspended_state_for_new_runtime_contexts() {
    let database = Database::new();
    let server = Server::start(None).await;
    let http = Client::builder().redirect(Policy::none()).build().unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let fixture = Fixture::new();
    let app = fixture.app_with_installations(
        database.store.clone(),
        Arc::new(AppInstallationClient::new(http, api, Arc::new(FixedJwt))),
    );
    app.installation(event(InstallationAction::Created))
        .await
        .unwrap();
    let original = database
        .store
        .organizations_by_github_id(17)
        .unwrap()
        .remove(0);
    let scale_set = database
        .store
        .scale_sets(original.workspace_id, original.id)
        .unwrap()
        .into_iter()
        .find(|set| set.github_id == 53)
        .unwrap();
    let target = ListenerTarget {
        workspace_id: original.workspace_id,
        organization_id: original.id,
        scale_set_id: scale_set.id,
    };
    let adapter = StoreAdapter::new(
        database.store.clone(),
        Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600))),
    );
    for (login, suspended, action) in [
        ("renamed-active", false, None),
        ("renamed-suspended", true, Some(InstallationAction::Suspend)),
        ("renamed-paused", true, None),
        (
            "renamed-resumed",
            false,
            Some(InstallationAction::Unsuspend),
        ),
    ] {
        *server.state.resource.organization_login.lock().unwrap() = login.to_owned();
        server
            .state
            .resource
            .suspended
            .store(suspended, Ordering::SeqCst);
        let synchronized = if let Some(action) = action {
            app.installation(event(action)).await.map(|_| ())
        } else {
            app.refresh_repositories(19).await
        };
        assert!(matches!(synchronized, Ok(())), "{login}: {synchronized:?}");
        let context = adapter.context(target).await.unwrap();
        let mut expected = original.clone();
        expected.login = login.to_owned();
        expected.app_state = if suspended {
            AppState::Suspended
        } else {
            AppState::Active
        };
        assert_eq!(context.organization, expected, "{login}");
        let sdk = Factory(server.origin.clone())
            .for_organization(&context.organization)
            .await
            .unwrap();
        let jit = sdk.register(53, "rename-runner").await.unwrap();
        assert_eq!(jit.runner_id, 71);
        assert!(
            server
                .state
                .requests
                .lock()
                .unwrap()
                .iter()
                .any(|(method, path, _)| *method == Method::POST
                    && path.contains(&format!("/orgs/{login}/actions/runners/registration-token"))),
            "{login}"
        );
        if suspended || action.is_some() {
            let output = fixture.output.0.lock().unwrap().last().unwrap().clone();
            let expected_output = if suspended {
                LifecycleEvent::Suspended(expected)
            } else {
                LifecycleEvent::Active(expected)
            };
            assert_eq!(output, expected_output, "{login}");
        }
    }
    assert_eq!(*fixture.authority.admins.lock().unwrap(), vec![41]);
    let saved = adapter.context(target).await.unwrap().organization;
    *server.state.resource.organization_login.lock().unwrap() = "different-account".to_owned();
    server.state.resource.account_id.store(18, Ordering::SeqCst);
    assert!(matches!(
        app.installation(event(InstallationAction::Suspend)).await,
        Err(error::Error::InstallationPending)
    ));
    assert_eq!(adapter.context(target).await.unwrap().organization, saved);
    server.state.resource.account_id.store(17, Ordering::SeqCst);
    let mut stale_event = event(InstallationAction::Unsuspend);
    stale_event.installation.id = 7;
    assert!(matches!(
        app.installation(stale_event).await,
        Err(error::Error::InstallationPending)
    ));
    assert_eq!(adapter.context(target).await.unwrap().organization, saved);
    let snapshot = GithubInstallationState {
        github_id: saved.github_id,
        installation_id: saved.installation_id,
        login: "must-not-be-saved".to_owned(),
        suspended: true,
    };
    let mut stale = snapshot.clone();
    stale.installation_id = 7;
    let mut different_account = snapshot.clone();
    different_account.github_id = 18;
    for (workspace, state) in [
        (id(999), snapshot.clone()),
        (saved.workspace_id, stale),
        (saved.workspace_id, different_account),
    ] {
        assert!(matches!(
            database
                .store
                .save_github_installation_state(workspace, saved.id, &state),
            Err(Error::NotFound)
        ));
        assert_eq!(adapter.context(target).await.unwrap().organization, saved);
    }
    database
        .store
        .record_app_removal(saved.workspace_id, saved.id, super::app_fixture::at())
        .unwrap();
    let removed = adapter.context(target).await.unwrap().organization;
    assert!(matches!(
        database
            .store
            .save_github_installation_state(saved.workspace_id, saved.id, &snapshot),
        Err(Error::NotFound)
    ));
    assert_eq!(adapter.context(target).await.unwrap().organization, removed);
}
