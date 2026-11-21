//! Declined requests, deleted identities, and concurrent browser reauthorization.

use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Duration;
use ciproxy_api::router::router;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, callback, cookie, json, oauth_state};

async fn reauthorize(
    fixture: &Fixture,
    browser: &Browser,
    outcome: CallbackOutcome,
) -> axum::response::Response {
    let begin = fixture
        .request(Some(browser), "GET", "/auth/github", None)
        .await;
    let binding = cookie(&begin, "__Host-ciproxy-oauth");
    let nonce = oauth_state(&begin);
    let request = Request::builder()
        .uri(format!(
            "/auth/github/callback?code=synthetic&state={nonce}"
        ))
        .header(
            header::COOKIE,
            format!(
                "__Host-ciproxy-oauth={binding}; __Host-ciproxy={}",
                browser.cookie
            ),
        )
        .body(Body::empty())
        .unwrap();
    callback(router(fixture.state.clone()), request, outcome).await
}

#[tokio::test]
async fn hourly_refresh_does_not_recreate_a_declined_request_in_another_workspace() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let other = Uuid::new_v4();
    fixture
        .database
        .store
        .create_workspace(&Workspace {
            id: other,
            name: "Other workspace".to_owned(),
            delete_after: None,
        })
        .unwrap();
    let mut organization = fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.workspace_id = other;
    organization.github_id = 300;
    organization.installation_id = 301;
    fixture
        .database
        .store
        .install_organization(&organization)
        .unwrap();
    fixture.identity.memberships.lock().unwrap().insert(
        8,
        vec![
            Membership {
                github_id: 100,
                owner: false,
            },
            Membership {
                github_id: 300,
                owner: false,
            },
        ],
    );
    let browser = fixture.login(8, Some(Role::Viewer)).await;
    fixture
        .database
        .store
        .save_dashboard_role(
            other,
            8,
            Some(Role::Viewer),
            "2026-11-06T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &fixture.path("/members/8"), None)
            .await
            .status(),
        StatusCode::OK
    );
    fixture.advance(Duration::hours(1));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await
            .status(),
        StatusCode::OK
    );
    assert!(
        !fixture
            .database
            .store
            .dashboard_members(fixture.workspace)
            .unwrap()
            .iter()
            .any(|member| member.github_user_id == 8)
    );
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(8)
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn reauthorization_handles_two_tabs_and_a_lost_callback_without_extending_signin() {
    let fixture = Fixture::new();
    fixture
        .identity
        .memberships
        .lock()
        .unwrap()
        .insert(77, Vec::new());
    let pending = fixture.login(77, None).await;
    fixture.advance(Duration::days(3));
    fixture.identity.memberships.lock().unwrap().insert(
        77,
        vec![Membership {
            github_id: 100,
            owner: false,
        }],
    );
    fixture.identity.id.store(77, Ordering::SeqCst);
    let (first, second) = tokio::join!(
        reauthorize(&fixture, &pending, CallbackOutcome::Accepted),
        reauthorize(&fixture, &pending, CallbackOutcome::Accepted)
    );
    let _lost = first;
    let current = Browser {
        cookie: cookie(&second, "__Host-ciproxy"),
        csrf: cookie(&second, "__Host-ciproxy-csrf"),
    };
    let response = fixture
        .request(Some(&pending), "GET", "/session", None)
        .await;
    assert_eq!(json(response).await["reauthorization_required"], true);
    let recovered = reauthorize(&fixture, &pending, CallbackOutcome::Accepted).await;
    let recovered = Browser {
        cookie: cookie(&recovered, "__Host-ciproxy"),
        csrf: cookie(&recovered, "__Host-ciproxy-csrf"),
    };
    assert_eq!(
        json(
            fixture
                .request(Some(&recovered), "GET", "/session", None)
                .await
        )
        .await["expires_at"],
        "2026-12-06T12:00:00Z"
    );
    let stale = fixture
        .request(Some(&current), "GET", "/session", None)
        .await;
    assert_eq!(stale.status(), StatusCode::OK);
}

#[tokio::test]
async fn owner_removal_reauthorization_shows_retention_without_private_workspace_data() {
    let fixture = Fixture::new();
    let browser = fixture.login(7, Some(Role::Admin)).await;
    fixture
        .database
        .store
        .record_app_removal(
            fixture.workspace,
            fixture.organization.id,
            "2026-11-06T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    fixture.advance(Duration::hours(1));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let response = reauthorize(&fixture, &browser, CallbackOutcome::Accepted).await;
    let browser = Browser {
        cookie: cookie(&response, "__Host-ciproxy"),
        csrf: cookie(&response, "__Host-ciproxy-csrf"),
    };
    let session = json(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(session["state"], "app_removed");
    assert_eq!(session["removed_organization"]["login"], "customer");
    assert_eq!(
        session["removed_organization"]["delete_after"],
        "2026-12-06T12:00:00Z"
    );
    assert_eq!(session["workspaces"].as_array().unwrap().len(), 0);
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .is_none()
    );
}
