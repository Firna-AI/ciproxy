//! Real OAuth/store/App composition for installation association scenarios.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::{
    Account, AccountKind, Actor, Installation, InstallationAction, InstallationEvent,
};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::workspaces::Workspace;
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use super::adapter_server::Server;
use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, json};
use super::setup_support::{browser, configure, finish};

pub(super) async fn scenario() -> (Fixture, Server, Browser, Organization) {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let mut added = fixture.organization.clone();
    added.id = Uuid::new_v4();
    added.github_id = 106;
    added.installation_id = 1066;
    added.login = "not-registered".to_owned();
    fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(added.clone());
    (fixture, server, admin, added)
}

pub(super) fn event(
    organization: &Organization,
    requester: i64,
    action: InstallationAction,
) -> InstallationEvent {
    InstallationEvent {
        action,
        installation: Installation {
            id: organization.installation_id,
            account: Account {
                id: organization.github_id,
                login: organization.login.clone(),
                kind: AccountKind::Organization,
            },
        },
        requester: Some(Actor { id: requester }),
        sender: Actor { id: 8 },
    }
}

pub(super) async fn begin(fixture: &Fixture, browser: &Browser) -> String {
    let response = fixture
        .request(
            Some(browser),
            "POST",
            &fixture.path("/organizations/add"),
            None,
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    install_state(json(response).await)
}

pub(super) async fn start(fixture: &Fixture, browser: &Browser) -> String {
    let response = fixture
        .request(Some(browser), "POST", "/installation-links/start", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    install_state(json(response).await)
}

fn install_state(link: Value) -> String {
    let url = Url::parse(link["url"].as_str().unwrap()).unwrap();
    assert!(url.path().ends_with("/installations/new"));
    assert_eq!(
        url.query_pairs()
            .map(|(key, _)| key.into_owned())
            .collect::<Vec<_>>(),
        vec!["state"]
    );
    url.query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned()
}

pub(super) async fn returned(
    fixture: &Fixture,
    state: &str,
    expected: CallbackOutcome,
) -> Option<Browser> {
    return_install(fixture, state, 1066, expected).await
}

pub(super) async fn return_install(
    fixture: &Fixture,
    state: &str,
    installation: i64,
    expected: CallbackOutcome,
) -> Option<Browser> {
    let begin = fixture
        .request(
            None,
            "GET",
            &format!("/github/setup?installation_id={installation}&state={state}"),
            None,
        )
        .await;
    let response = finish(fixture, begin, expected).await;
    match expected {
        CallbackOutcome::Accepted => Some(browser(&response)),
        CallbackOutcome::Refused => None,
    }
}

pub(super) async fn claims(fixture: &Fixture, browser: &Browser) -> Value {
    let response = fixture
        .request(Some(browser), "GET", "/installation-links", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    json(response).await
}

pub(super) fn other_workspace(fixture: &Fixture) -> Workspace {
    let workspace = Workspace {
        id: Uuid::new_v4(),
        name: "Other workspace".to_owned(),
        delete_after: None,
    };
    fixture.database.store.create_workspace(&workspace).unwrap();
    let mut organization = fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.workspace_id = workspace.id;
    organization.github_id = 200;
    organization.installation_id = 201;
    organization.login = "other-customer".to_owned();
    fixture
        .database
        .store
        .install_organization(&organization)
        .unwrap();
    fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(organization);
    fixture
        .database
        .store
        .save_dashboard_role(workspace.id, 7, Some(Role::Admin), fixture.runtime.now())
        .unwrap();
    workspace
}

pub(super) fn tier(fixture: &Fixture, workspace: Uuid, tier: Tier) {
    let mut session = fixture
        .database
        .store
        .try_lock_billing(workspace)
        .unwrap()
        .unwrap();
    let mut record = session.record().unwrap().state;
    record.tier = tier;
    session
        .commit(Some(&record), None, false, false, fixture.runtime.now())
        .unwrap();
}

pub(super) async fn maintenance(fixture: &Fixture) {
    Maintenance {
        state: fixture.state.clone(),
    }
    .reconcile()
    .await
    .unwrap();
}

pub(super) fn memberships(server: &Server, ids: &[i64]) {
    *server.control.membership_ids.lock().unwrap() = Some(ids.to_vec());
}

pub(super) async fn second_admin(fixture: &Fixture, server: &Server) -> Browser {
    memberships(server, &[100, 106, 200]);
    server.control.person.store(8, Ordering::SeqCst);
    fixture.identity.memberships.lock().unwrap().insert(
        8,
        vec![Membership {
            github_id: 100,
            owner: false,
        }],
    );
    fixture.login(8, Some(Role::Admin)).await
}

pub(super) fn pending(outcome: InstallationOutcome) -> Uuid {
    assert!(
        matches!(outcome, InstallationOutcome::AwaitingAssignment { .. }),
        "expected a durable unassigned installation: {outcome:?}"
    );
    match outcome {
        InstallationOutcome::AwaitingAssignment { claim_id } => claim_id,
        other => panic!("expected pending claim, got {other:?}"),
    }
}
