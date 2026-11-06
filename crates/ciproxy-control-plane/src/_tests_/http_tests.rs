//! The single HTTP mount keeps health, dashboard, and hook routes separate.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use ciproxy_github::hooks::{WebhookHandlerMock, WebhookOutcome};
use tower::ServiceExt;
use unimock::{MockFn, Unimock, matching};

use super::router;

#[tokio::test]
async fn readiness_follows_runtime_state_on_the_shared_api_host() {
    let ready = Arc::new(AtomicBool::new(false));
    let app = router(
        Router::new(),
        Router::new().route("/probe", get(|| async { StatusCode::ACCEPTED })),
        Arc::new(Unimock::new(())),
        ready.clone(),
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    ready.store(true, Ordering::Release);
    for (path, status) in [
        ("/api/health/ready", StatusCode::OK),
        ("/api/health/live", StatusCode::OK),
        ("/api/probe", StatusCode::ACCEPTED),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{path}");
    }
}

#[tokio::test]
async fn github_hook_passes_the_original_body_and_signature_to_its_verifier() {
    let hooks = Arc::new(Unimock::new(
        WebhookHandlerMock::handle
            .some_call(matching!((event, signature, body) if *event == "ping" && *signature == "sha256=fixture" && *body == b"original bytes"))
            .returns(Ok(WebhookOutcome::Ignored)),
    ));
    let app = router(
        Router::new(),
        Router::new(),
        hooks.clone(),
        Arc::new(AtomicBool::new(true)),
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/hooks/github")
                .header("x-github-event", "ping")
                .header("x-hub-signature-256", "sha256=fixture")
                .body(Body::from("original bytes"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn valid_provider_sized_webhook_is_not_rejected_by_the_framework_limit() {
    let bytes = vec![b' '; 25 * 1024 * 1024];
    let hooks = Arc::new(Unimock::new(
        WebhookHandlerMock::handle
            .some_call(matching!((_,_,body) if body.len()==25*1024*1024))
            .returns(Ok(WebhookOutcome::Ignored)),
    ));
    let app = router(
        Router::new(),
        Router::new(),
        hooks.clone(),
        Arc::new(AtomicBool::new(true)),
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/hooks/github")
                .header("x-github-event", "ping")
                .header("x-hub-signature-256", "sha256=fixture")
                .body(Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
