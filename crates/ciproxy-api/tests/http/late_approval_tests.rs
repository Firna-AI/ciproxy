//! Fresh sign-in names exact late approvals without choosing completed history.

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

use chrono::Duration;
use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;

use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{begin, event, memberships, other_workspace, pending, scenario};
use super::setup_support::{browser, finish};

fn historical_claim(fixture: &Fixture) -> Uuid {
    let claim = InstallationClaim {
        id: Uuid::new_v4(),
        installation_id: fixture.organization.installation_id,
        github_id: fixture.organization.github_id,
        login: fixture.organization.login.clone(),
        requester_id: 7,
        event_seen: true,
        user_verified: true,
        direct: true,
        workspace_id: Some(fixture.workspace),
        organization_id: Some(fixture.organization.id),
        verified_at: Some(fixture.runtime.now()),
    };
    fixture
        .database
        .store
        .save_installation_claim(&claim, None, InstallationResultDelivery::Untracked)
        .unwrap()
        .id
}

async fn session(fixture: &Fixture, browser: &Browser) -> Value {
    json(
        fixture
            .request(Some(browser), "GET", "/session", None)
            .await,
    )
    .await
}

async fn authorize(fixture: &Fixture, prior: Option<&Browser>) -> Browser {
    browser(
        &finish(
            fixture,
            fixture.request(prior, "GET", "/auth/github", None).await,
            CallbackOutcome::Accepted,
        )
        .await,
    )
}

async fn waiting(fixture: &Fixture, prior: &Browser, state: &str) -> Browser {
    browser(
        &finish(
            fixture,
            fixture
                .request(
                    Some(prior),
                    "GET",
                    &format!("/github/setup?setup_action=request&state={state}"),
                    None,
                )
                .await,
            CallbackOutcome::Accepted,
        )
        .await,
    )
}

fn continuation(session: &Value) -> BTreeSet<String> {
    session["installation_continuation"]
        .as_array()
        .expect("session must expose exact installation continuation")
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn late_first_install_sign_in_names_new_direct_claim_and_preserves_viewer_workspace() {
    let (fixture, server, prior, added) = scenario().await;
    let old = historical_claim(&fixture);
    fixture
        .database
        .store
        .save_dashboard_role(
            fixture.workspace,
            7,
            Some(Role::Viewer),
            fixture.runtime.now(),
        )
        .unwrap();
    let initial = session(&fixture, &prior).await;
    let url = Url::parse(initial["install_url"].as_str().unwrap()).unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    let prior = waiting(&fixture, &prior, &state).await;
    assert_eq!(
        session(&fixture, &prior).await["setup"]["kind"],
        "waiting_approval"
    );
    fixture.advance(Duration::days(2));
    let approved = fixture
        .app
        .installation(event(&added, 7, InstallationAction::Created))
        .await
        .unwrap();
    assert!(matches!(approved, InstallationOutcome::Organization(_)));
    let linked = fixture
        .database
        .store
        .claim_for_installation(1066)
        .unwrap()
        .unwrap();
    assert!(!linked.user_verified);
    server
        .control
        .fail_user_installations
        .store(true, Ordering::SeqCst);
    let unavailable = authorize(&fixture, Some(&prior)).await;
    assert_eq!(
        session(&fixture, &unavailable).await["installation_continuation"],
        json!([])
    );
    server
        .control
        .fail_user_installations
        .store(false, Ordering::SeqCst);
    let current = authorize(&fixture, Some(&unavailable)).await;
    let resolved = session(&fixture, &current).await;
    assert_eq!(resolved["installation_continuation"], json!([linked.id]));
    assert_eq!(
        continuation(&resolved),
        BTreeSet::from([linked.id.to_string()])
    );
    assert!(!continuation(&resolved).contains(&old.to_string()));
    assert_eq!(resolved["expires_at"], initial["expires_at"]);
    assert_eq!(resolved["setup"]["kind"], "waiting_approval");
    assert_eq!(
        continuation(&session(&fixture, &current).await),
        continuation(&resolved)
    );
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(fixture.workspace, 7)
            .unwrap(),
        Some(Role::Viewer)
    );
    assert_ne!(linked.workspace_id, Some(fixture.workspace));
    let next = authorize(&fixture, Some(&current)).await;
    assert_eq!(
        session(&fixture, &next).await["installation_continuation"],
        json!([])
    );
    let browserless = authorize(&fixture, None).await;
    assert_eq!(
        session(&fixture, &browserless).await["installation_continuation"],
        json!([])
    );
    server.control.person.store(8, Ordering::SeqCst);
    let other_person = authorize(&fixture, Some(&current)).await;
    assert_eq!(
        session(&fixture, &other_person).await["installation_continuation"],
        json!([])
    );
}

#[tokio::test]
async fn late_add_sign_in_names_all_resolved_claims_and_ignores_completed_viewer_history() {
    let (fixture, server, prior, added) = scenario().await;
    let old = historical_claim(&fixture);
    let destination = other_workspace(&fixture);
    let state = begin(&fixture, &prior).await;
    let prior = waiting(&fixture, &prior, &state).await;
    fixture
        .database
        .store
        .save_dashboard_role(
            fixture.workspace,
            7,
            Some(Role::Viewer),
            fixture.runtime.now(),
        )
        .unwrap();
    memberships(&server, &[100, 106, 200, 206]);
    let mut second = added.clone();
    second.id = Uuid::new_v4();
    second.github_id = 206;
    second.installation_id = 2066;
    second.login = "second-late-approval".to_owned();
    fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(second.clone());
    server.control.extra_installations.lock().unwrap().push((
        second.installation_id,
        second.github_id,
        second.login.clone(),
    ));
    let first = pending(
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap(),
    );
    let second = pending(
        fixture
            .app
            .installation(event(&second, 7, InstallationAction::Created))
            .await
            .unwrap(),
    );
    let current = authorize(&fixture, Some(&prior)).await;
    let resolved = session(&fixture, &current).await;
    assert_eq!(
        resolved["installation_continuation"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        continuation(&resolved),
        BTreeSet::from([first.to_string(), second.to_string()])
    );
    assert!(!continuation(&resolved).contains(&old.to_string()));
    let links = resolved["organization_links"].as_array().unwrap();
    assert_eq!(links.len(), 3);
    for id in [first, second] {
        let link = links
            .iter()
            .find(|link| link["id"] == id.to_string())
            .unwrap();
        assert_eq!(link["workspace_id"], destination.id.to_string());
        assert_eq!(link["state"], "linked_inactive");
        assert_eq!(link["activation"]["reasons"], json!(["billing_upgrade"]));
    }
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(fixture.workspace, 7)
            .unwrap(),
        Some(Role::Viewer)
    );
    assert_eq!(
        continuation(&session(&fixture, &current).await),
        continuation(&resolved)
    );
    let next = authorize(&fixture, Some(&current)).await;
    assert_eq!(
        session(&fixture, &next).await["installation_continuation"],
        json!([])
    );
    let browserless = authorize(&fixture, None).await;
    assert_eq!(
        session(&fixture, &browserless).await["installation_continuation"],
        json!([])
    );
}
