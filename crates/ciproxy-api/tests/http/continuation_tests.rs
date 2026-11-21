//! Exact authorization continuations remain independent of browser cookie size.

use std::collections::BTreeSet;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use serde_json::{Value, json};
use uuid::Uuid;

use super::adapter_server::Server;
use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, cookie, json};
use super::link_support::memberships;
use super::setup_support::{browser, configure, finish};

async fn session(fixture: &Fixture, browser: &Browser) -> Value {
    let response = fixture
        .request(Some(browser), "GET", "/session", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    json(response).await
}

fn continuation(session: &Value) -> BTreeSet<Uuid> {
    serde_json::from_value(session["installation_continuation"].clone()).unwrap()
}

#[tokio::test]
async fn hundred_open_claims_keep_exact_continuation_with_bounded_cookies_and_fixed_expiry() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    memberships(&server, &(30000..30100).collect::<Vec<_>>());
    let prior = fixture.login(7, None).await;
    let initial = session(&fixture, &prior).await;
    assert_eq!(initial["state"], "install_required");
    let mut expected = BTreeSet::new();
    for index in 0..100 {
        let claim = InstallationClaim {
            id: Uuid::new_v4(),
            installation_id: 50000 + index,
            github_id: 30000 + index,
            login: format!("waiting-{index}"),
            requester_id: 7,
            event_seen: true,
            user_verified: false,
            direct: false,
            workspace_id: None,
            organization_id: None,
            verified_at: None,
        };
        expected.insert(claim.id);
        server.control.extra_installations.lock().unwrap().push((
            claim.installation_id,
            claim.github_id,
            claim.login.clone(),
        ));
        fixture
            .database
            .store
            .save_installation_claim(&claim, None, InstallationResultDelivery::Untracked)
            .unwrap();
    }
    let current = browser(
        &finish(
            &fixture,
            fixture
                .request(Some(&prior), "GET", "/auth/github", None)
                .await,
            CallbackOutcome::Accepted,
        )
        .await,
    );
    assert_eq!(current.cookie.len(), prior.cookie.len());
    assert!(current.cookie.len() <= 3800);
    let resolved = session(&fixture, &current).await;
    assert_eq!(continuation(&resolved), expected);
    assert_eq!(resolved["state"], "install_required");
    assert_eq!(resolved["workspaces"], json!([]));
    assert_eq!(resolved["expires_at"], initial["expires_at"]);
    assert_eq!(continuation(&session(&fixture, &current).await), expected);
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .is_none()
    );

    let claims =
        serde_json::to_value(fixture.database.store.installation_claims(7).unwrap()).unwrap();
    finish(
        &fixture,
        fixture
            .request(
                Some(&current),
                "GET",
                "/github/setup?installation_id=201",
                None,
            )
            .await,
        CallbackOutcome::Refused,
    )
    .await;
    assert_eq!(
        serde_json::to_value(fixture.database.store.installation_claims(7).unwrap()).unwrap(),
        claims
    );
    assert_eq!(continuation(&session(&fixture, &current).await), expected);

    fixture.advance(Duration::days(3));
    let begin = fixture
        .request(Some(&current), "GET", "/auth/github", None)
        .await;
    let binding_length = cookie(&begin, "__Host-ciproxy-oauth").len();
    assert!(binding_length <= 3800);
    println!(
        "claims=100 session_cookie_bytes={} oauth_binding_bytes={binding_length}",
        current.cookie.len()
    );
    let repeated = browser(&finish(&fixture, begin, CallbackOutcome::Accepted).await);
    let repeated_session = session(&fixture, &repeated).await;
    assert_eq!(continuation(&repeated_session), expected);
    assert_eq!(repeated_session["expires_at"], initial["expires_at"]);

    server.control.extra_installations.lock().unwrap().clear();
    let empty = browser(
        &finish(
            &fixture,
            fixture
                .request(Some(&repeated), "GET", "/auth/github", None)
                .await,
            CallbackOutcome::Accepted,
        )
        .await,
    );
    assert_eq!(empty.cookie.len(), current.cookie.len());
    let empty_session = session(&fixture, &empty).await;
    assert_eq!(empty_session["installation_continuation"], json!([]));
    assert_eq!(empty_session["expires_at"], initial["expires_at"]);
    assert_eq!(
        session(&fixture, &repeated).await["installation_continuation"],
        json!([])
    );
    let begin = fixture
        .request(Some(&empty), "GET", "/auth/github", None)
        .await;
    assert_eq!(cookie(&begin, "__Host-ciproxy-oauth").len(), binding_length);
}
