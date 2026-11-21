//! Copied commands select the issuing control plane from trusted configuration.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use ciproxy_api::router::router;
use ciproxy_store_interface::dashboard::records::Role;
use tower::ServiceExt;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn preview_install_command_uses_configured_origin_despite_request_headers() {
    let mut fixture = Fixture::new();
    fixture.state.config.public_origin = "https://preview.ciproxy.example:8443".to_owned();
    fixture.state.config.download_url = "https://get.preview.ciproxy.example".to_owned();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let request = Request::builder()
        .method("POST")
        .uri(fixture.path("/install-tokens"))
        .header(header::COOKIE, format!("__Host-ciproxy={}", admin.cookie))
        .header(header::ORIGIN, &fixture.state.config.public_origin)
        .header("x-ciproxy-csrf", &admin.csrf)
        .header(header::HOST, "untrusted.example")
        .header("x-forwarded-host", "untrusted.example")
        .header("x-forwarded-proto", "http")
        .body(Body::empty())
        .unwrap();
    let response = router(fixture.state.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let command = json(response).await;
    let command = command["command"].as_str().unwrap();
    let (prefix, token) = command.split_once(" --token ").unwrap();
    assert_eq!(
        prefix,
        "curl -fsSL https://get.preview.ciproxy.example | sudo sh -s -- install --agent-url 'wss://preview.ciproxy.example:8443/agent/'"
    );
    assert!(!token.is_empty());
}

#[tokio::test]
async fn production_install_command_keeps_the_documented_form() {
    let mut fixture = Fixture::new();
    fixture.state.config.public_origin = "https://app.ciproxy.dev".to_owned();
    fixture.state.config.download_url = "https://get.ciproxy.dev".to_owned();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let response = fixture
        .request(Some(&admin), "POST", &fixture.path("/install-tokens"), None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let command = json(response).await;
    let command = command["command"].as_str().unwrap();
    let (prefix, token) = command.split_once(" --token ").unwrap();
    assert_eq!(
        prefix,
        "curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install"
    );
    assert!(!token.is_empty());
}
