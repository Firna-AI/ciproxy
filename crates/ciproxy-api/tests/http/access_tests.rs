//! HTTP sign-in, role isolation, retention, and owner recovery requirements.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use ciproxy_api::router::router;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use tower::ServiceExt;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn roles_gate_settings_and_mutations_and_members_ask_for_access() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let pending = fixture.login(8, None).await;
    let session = json(
        fixture
            .request(Some(&pending), "GET", "/session", None)
            .await,
    )
    .await;
    assert!(session["workspaces"][0]["role"].is_null());
    assert_eq!(session["workspaces"][0]["admins"], json!(["person-7"]));
    assert_eq!(
        fixture
            .request(Some(&pending), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let members = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/members"), None)
            .await,
    )
    .await;
    assert_eq!(members["open_requests"], 1);
    assert!(
        members["admin_notice"]
            .as_str()
            .unwrap()
            .contains("every organization")
    );
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &fixture.path("/members/8"),
                Some(json!({"role":"viewer"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .request(Some(&pending), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::OK
    );
    for role in ["viewer", "billing"] {
        assert_eq!(
            fixture
                .request(
                    Some(&admin),
                    "PUT",
                    &fixture.path("/members/8"),
                    Some(json!({"role":role}))
                )
                .await
                .status(),
            StatusCode::OK
        );
        for suffix in [
            "/settings",
            "/labels",
            "/members",
            "/organizations",
            "/setup",
        ] {
            assert_eq!(
                fixture
                    .request(Some(&pending), "GET", &fixture.path(suffix), None)
                    .await
                    .status(),
                StatusCode::FORBIDDEN
            );
        }
        assert_eq!(
            fixture
                .request(
                    Some(&pending),
                    "POST",
                    &fixture.path("/install-tokens"),
                    None
                )
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    let request = Request::builder()
        .method("PUT")
        .uri(fixture.path("/settings"))
        .header(header::COOKIE, format!("__Host-ciproxy={}", admin.cookie))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{\"name\":\"Intruder\"}"))
        .unwrap();
    let denied = router(fixture.state.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    assert_eq!(json(denied).await["code"], "access_denied");
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &fixture.path("/members/8"), None)
            .await
            .status(),
        StatusCode::OK
    );
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(8)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        json(
            fixture
                .request(Some(&pending), "GET", "/session", None)
                .await
        )
        .await["state"],
        "access_ended"
    );
}
