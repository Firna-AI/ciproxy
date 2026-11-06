//! HTTP sign-in, role isolation, retention, and owner recovery requirements.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Duration;
use ciproxy_api::router::router;
use ciproxy_store_interface::dashboard::records::Role;
use tower::ServiceExt;

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
    assert_eq!(
        fixture
            .request(Some(&browser), "POST", "/auth/sign-out", None)
            .await
            .status(),
        StatusCode::OK
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
