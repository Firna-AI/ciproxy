//! Setup rejects spoofed identity and preserves explicit install continuation.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::Role;

use super::adapter_server::Server;
use super::fixture::Fixture;
use super::fixture_io::{CallbackOutcome, assert_callback, json};
use super::setup_support::{begin, browser, configure, finish, old_callback_status};

#[tokio::test]
async fn setup_proves_organization_and_preserves_start_bound_expiry() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    server.control.memberships.store(false, Ordering::SeqCst);
    let prior = fixture.login(7, None).await;
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .is_none()
    );
    fixture.advance(Duration::days(2));
    server.control.memberships.store(true, Ordering::SeqCst);
    let response = finish(
        &fixture,
        begin(
            &fixture,
            Some(&prior),
            "?installation_id=101&state=forged&workspace_id=forged",
        )
        .await,
        CallbackOutcome::Accepted,
    )
    .await;
    let signed = browser(&response);
    let session = json(
        fixture
            .request(Some(&signed), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(session["expires_at"], "2026-12-06T12:00:00Z");
    assert_eq!(session["setup"]["kind"], "organization");
    assert_eq!(session["setup"]["installation_id"], "101");
    assert_eq!(session["setup"]["github_id"], "100");
    assert_eq!(
        session["workspaces"][0]["id"],
        fixture.workspace.to_string()
    );
    assert!(session["workspaces"][0]["role"].is_null());
    assert_eq!(
        fixture
            .database
            .store
            .organization_by_installation_id(101)
            .unwrap()
            .unwrap()
            .workspace_id,
        fixture.workspace
    );
    let unchanged = json(
        fixture
            .request(Some(&signed), "GET", "/session?installation_id=700", None)
            .await,
    )
    .await;
    assert_eq!(unchanged["setup"]["kind"], "organization");
    let response = finish(
        &fixture,
        fixture
            .request(
                Some(&signed),
                "GET",
                "/github/setup?installation_id=1066",
                None,
            )
            .await,
        CallbackOutcome::Accepted,
    )
    .await;
    let next = browser(&response);
    assert_eq!(
        json(fixture.request(Some(&next), "GET", "/session", None).await).await["setup"]["github_id"],
        "106"
    );
    assert!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .is_none()
    );
    assert_eq!(old_callback_status(&fixture).await, StatusCode::NOT_FOUND);
    for path in ["/auth/callback", "/api/auth/callback"] {
        assert_eq!(
            fixture.request(None, "GET", path, None).await.status(),
            StatusCode::NOT_FOUND
        );
    }
}

#[tokio::test]
async fn setup_spoofs_and_unavailable_facts_leave_existing_business_state_unchanged() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    let signed = fixture.login(7, Some(Role::Viewer)).await;
    let before = fixture
        .database
        .store
        .dashboard_user_by_github(7)
        .unwrap()
        .unwrap();
    fixture.advance(Duration::hours(1));
    for target in [201, 701, 702, 703, 704, 705] {
        finish(
            &fixture,
            fixture
                .request(
                    Some(&signed),
                    "GET",
                    &format!("/github/setup?installation_id={target}"),
                    None,
                )
                .await,
            CallbackOutcome::Refused,
        )
        .await;
        let after = fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .unwrap();
        assert_eq!(after.checked_at, before.checked_at);
        assert!(after.credentials == before.credentials);
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_members(fixture.workspace)
                .unwrap()
                .len(),
            1
        );
    }
    server
        .control
        .fail_user_installations
        .store(true, Ordering::SeqCst);
    finish(
        &fixture,
        fixture
            .request(
                Some(&signed),
                "GET",
                "/github/setup?installation_id=101",
                None,
            )
            .await,
        CallbackOutcome::Refused,
    )
    .await;
    server
        .control
        .fail_user_installations
        .store(false, Ordering::SeqCst);
    for path in [
        "/github/setup",
        "/github/setup?installation_id=invalid",
        "/github/setup?installation_id=9999",
    ] {
        let response = fixture.request(Some(&signed), "GET", path, None).await;
        assert_callback(&response, CallbackOutcome::Refused);
    }
    server
        .control
        .fail_app_installations
        .store(true, Ordering::SeqCst);
    let response = fixture
        .request(
            Some(&signed),
            "GET",
            "/github/setup?installation_id=101",
            None,
        )
        .await;
    assert_callback(&response, CallbackOutcome::Refused);
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .unwrap()
            .checked_at,
        before.checked_at
    );
    assert!(
        fixture
            .database
            .store
            .organizations_by_github_id(200)
            .unwrap()
            .is_empty()
    );
}
