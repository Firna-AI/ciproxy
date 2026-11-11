//! Existing workspace authority and browser protections cover every new page.

use axum::http::{StatusCode, header};
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use super::fixture_io;
use super::m18_stripe::Stripe;
use super::m18_support::Pages;

#[tokio::test]
async fn m18_viewer_reads_pages_but_cannot_pay_or_cross_workspace_authority() {
    let pages = Pages::new(Stripe::new().provider());
    let viewer = pages.fixture.login(8, Some(Role::Viewer)).await;
    for route in [
        "/billing",
        "/plans",
        "/banner",
        "/savings",
        "/jobs/live",
        "/jobs/history",
    ] {
        assert_eq!(
            pages
                .request(Some(&viewer), "GET", route, None)
                .await
                .status(),
            StatusCode::OK,
            "{route}"
        );
        assert_eq!(
            pages.request(None, "GET", route, None).await.status(),
            StatusCode::UNAUTHORIZED,
            "{route}"
        );
    }
    let selection = json!({"tier":"team","add_ons":{"start_blocks":0,"machines":0}});
    for (method, route, body) in [
        ("POST", "/checkout", Some(selection.clone())),
        (
            "POST",
            "/checkout/return",
            Some(json!({"session_id":"cs_m18"})),
        ),
        (
            "POST",
            "/billing/change",
            Some(json!({"quote_id":Uuid::nil()})),
        ),
        (
            "PUT",
            "/billing/add-ons",
            Some(json!({"start_blocks":1,"machines":0})),
        ),
        ("POST", "/billing/move-to-free", None),
        ("POST", "/billing/portal", None),
    ] {
        assert_eq!(
            pages
                .request(Some(&viewer), method, route, body)
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "{route}"
        );
    }
    assert_eq!(
        pages
            .request(Some(&viewer), "GET", "/checkout/review", None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let mut request = pages.build(Some(&viewer), "GET", "/billing", None);
    *request.uri_mut() = format!("/api/workspaces/{}/billing", Uuid::new_v4())
        .parse()
        .unwrap();
    assert_eq!(
        pages.application().oneshot(request).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let admin = pages.fixture.login(7, Some(Role::Admin)).await;
    let mut request = pages.build(
        Some(&admin),
        "POST",
        "/checkout",
        Some(json!({"tier":"team","add_ons":{"start_blocks":0,"machines":0}})),
    );
    request.headers_mut().remove("x-ciproxy-csrf");
    let response = pages.application().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(fixture_io::json(response).await["code"], "access_denied");
    let mut request = pages.build(Some(&admin), "POST", "/billing/portal", None);
    request
        .headers_mut()
        .insert(header::ORIGIN, "https://other.example".parse().unwrap());
    assert_eq!(
        pages.application().oneshot(request).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
}
