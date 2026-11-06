//! HTTP sign-in, role isolation, retention, and owner recovery requirements.

use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Duration;
use ciproxy_api::router::router;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use tower::ServiceExt;

use super::fixture::Fixture;
use super::fixture_io::{cookie, json};

#[tokio::test]
async fn hourly_membership_loss_removes_role_person_and_existing_sessions() {
    let fixture = Fixture::new();
    let browser = fixture.login(7, Some(Role::Viewer)).await;
    fixture
        .identity
        .memberships
        .lock()
        .unwrap()
        .insert(7, Vec::new());
    fixture.advance(Duration::minutes(59));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::OK
    );
    fixture.advance(Duration::minutes(1));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .database
            .store
            .dashboard_members(fixture.workspace)
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn pending_browser_state_retains_no_person_and_promotes_without_extending_expiry() {
    let fixture = Fixture::new();
    fixture
        .identity
        .memberships
        .lock()
        .unwrap()
        .insert(77, Vec::new());
    let pending = fixture.login(77, None).await;
    assert!(pending.cookie.starts_with("v1."));
    assert!(pending.cookie.len() < 3800);
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(77)
            .unwrap()
            .is_none()
    );
    fixture.advance(Duration::days(2));
    fixture.identity.memberships.lock().unwrap().insert(
        77,
        vec![Membership {
            github_id: 100,
            owner: false,
        }],
    );
    let durable = fixture.login(77, Some(Role::Viewer)).await;
    let canonical = fixture
        .database
        .store
        .dashboard_user_by_github(77)
        .unwrap()
        .unwrap()
        .id;
    let response = fixture
        .request(Some(&pending), "GET", "/session", None)
        .await;
    let payload = json(response).await;
    assert_eq!(payload["reauthorization_required"], true);
    assert_eq!(payload["expires_at"], "2026-12-06T12:00:00Z");
    fixture.identity.id.store(77, Ordering::SeqCst);
    let begin = fixture.request(None, "GET", "/auth/github", None).await;
    let state = cookie(&begin, "__Host-ciproxy-oauth");
    let request = Request::builder()
        .uri(format!("/auth/callback?code=synthetic&state={state}"))
        .header(
            header::COOKIE,
            format!(
                "__Host-ciproxy-oauth={state}; __Host-ciproxy={}",
                pending.cookie
            ),
        )
        .body(Body::empty())
        .unwrap();
    let response = router(fixture.state.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let browser = super::fixture::Browser {
        cookie: cookie(&response, "__Host-ciproxy"),
        csrf: cookie(&response, "__Host-ciproxy-csrf"),
    };
    let promoted = json(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(promoted["state"], "ready");
    assert_eq!(promoted["expires_at"], "2026-12-06T12:00:00Z");
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_user_by_github(77)
            .unwrap()
            .unwrap()
            .id,
        canonical
    );
    assert_eq!(
        fixture
            .request(Some(&durable), "GET", "/session", None)
            .await
            .status(),
        StatusCode::OK
    );
}
