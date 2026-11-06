//! HTTP sign-in, role isolation, retention, and owner recovery requirements.

use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Duration;
use ciproxy_api::router::router;
use ciproxy_api::runtime::Runtime;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::installation_links::InstallationClaim;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::{CallbackOutcome, assert_callback, callback, cookie, json, oauth_state};

#[tokio::test]
async fn oauth_is_browser_bound_one_use_and_sessions_end_at_thirty_days() {
    let fixture = Fixture::new();
    let begin = fixture.request(None, "GET", "/auth/github", None).await;
    let binding = cookie(&begin, "__Host-ciproxy-oauth");
    let state = oauth_state(&begin);
    let bad = Request::builder()
        .uri(format!(
            "/auth/github/callback?code=synthetic&state={state}"
        ))
        .header(header::COOKIE, "__Host-ciproxy-oauth=another-browser")
        .body(Body::empty())
        .unwrap();
    callback(router(fixture.state.clone()), bad, CallbackOutcome::Refused).await;
    let valid = Request::builder()
        .uri(format!(
            "/auth/github/callback?code=synthetic&state={state}"
        ))
        .header(header::COOKIE, format!("__Host-ciproxy-oauth={binding}"))
        .body(Body::empty())
        .unwrap();
    callback(
        router(fixture.state.clone()),
        valid,
        CallbackOutcome::Accepted,
    )
    .await;
    let replay = Request::builder()
        .uri(format!("/auth/github/callback?code=replayed&state={state}"))
        .header(header::COOKIE, format!("__Host-ciproxy-oauth={binding}"))
        .body(Body::empty())
        .unwrap();
    callback(
        router(fixture.state.clone()),
        replay,
        CallbackOutcome::Refused,
    )
    .await;
    let mounted = Router::new().nest("/api", router(fixture.state.clone()));
    let response = mounted
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/github")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get("x-ciproxy-build").unwrap(),
        "http-test-build"
    );
    let cancelled = fixture
        .request(
            None,
            "GET",
            "/auth/github/callback?error=access_denied",
            None,
        )
        .await;
    assert_callback(&cancelled, CallbackOutcome::Refused);
    let browser = fixture.login(7, Some(Role::Admin)).await;
    let response = mounted
        .oneshot(
            Request::builder()
                .uri("/api/session")
                .header(header::COOKIE, format!("__Host-ciproxy={}", browser.cookie))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-ciproxy-build").unwrap(),
        "http-test-build"
    );
    let session = json(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(session["expires_at"], "2026-12-06T12:00:00Z");
    fixture.advance(Duration::days(30) - Duration::seconds(1));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await
            .status(),
        StatusCode::OK
    );
    fixture.advance(Duration::seconds(1));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let browser = fixture.login(7, Some(Role::Admin)).await;
    let claim = InstallationClaim {
        id: Uuid::new_v4(),
        installation_id: 1066,
        github_id: 106,
        login: "waiting-org".to_owned(),
        requester_id: 7,
        event_seen: true,
        user_verified: true,
        direct: false,
        workspace_id: None,
        organization_id: None,
        verified_at: Some(fixture.runtime.now()),
    };
    fixture
        .database
        .store
        .save_installation_claim(&claim, None)
        .unwrap();
    fixture
        .database
        .store
        .offer_installation_workspaces(claim.id, 7, &[fixture.workspace])
        .unwrap();
    assert_eq!(
        fixture
            .request(Some(&browser), "POST", "/auth/sign-out", None)
            .await
            .status(),
        StatusCode::OK
    );
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .is_some()
    );
    fixture.identity.fail.store(true, Ordering::SeqCst);
    for path in [
        "/installation-links".to_owned(),
        format!("/installation-links/{}", claim.id),
    ] {
        let response = fixture.request(Some(&browser), "GET", &path, None).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let data = json(response).await;
        let view = if path == "/installation-links" {
            &data[0]
        } else {
            &data
        };
        assert_eq!(view["state"], "reauthorization_required");
        assert_eq!(view["reauthorization_required"], true);
        assert_eq!(view["workspaces"], json!([]));
        assert!(view["activation"].is_null());
        assert!(view["workspace_id"].is_null());
    }
    let refused = fixture
        .request(
            Some(&browser),
            "PUT",
            &format!("/installation-links/{}/workspace", claim.id),
            Some(json!({"workspace_id":fixture.workspace})),
        )
        .await;
    assert_eq!(refused.status(), StatusCode::UNAUTHORIZED);
    assert!(
        fixture
            .database
            .store
            .installation_claim(claim.id)
            .unwrap()
            .unwrap()
            .workspace_id
            .is_none()
    );
    assert!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .is_none()
    );
    let signed_out = json(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(signed_out["state"], "access_ended");
    assert_eq!(signed_out["workspaces"].as_array().unwrap().len(), 0);
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
}
