//! HTTP workspace roles, isolated access, and independent browser-proof checks.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use ciproxy_api::router::router;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::workspaces::Workspace;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn roles_gate_settings_and_mutations_and_members_ask_for_access() {
    let fixture = Fixture::new();
    let second = Uuid::new_v4();
    fixture
        .database
        .store
        .create_workspace(&Workspace {
            id: second,
            name: "Other customer".to_owned(),
            delete_after: None,
        })
        .unwrap();
    let mut organization = fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.workspace_id = second;
    organization.github_id = 200;
    organization.installation_id = 201;
    organization.login = "other-customer".to_owned();
    fixture
        .database
        .store
        .install_organization(&organization)
        .unwrap();
    fixture.identity.memberships.lock().unwrap().insert(
        7,
        vec![
            Membership {
                github_id: 100,
                owner: false,
            },
            Membership {
                github_id: 200,
                owner: false,
            },
        ],
    );
    let admin = fixture.login(7, Some(Role::Admin)).await;
    assert_eq!(
        fixture.database.store.dashboard_role(second, 7).unwrap(),
        None
    );
    let other = format!("/workspaces/{second}/settings");
    assert_eq!(
        fixture
            .request(Some(&admin), "GET", &other, None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &other,
                Some(json!({"name":"Intruder"}))
            )
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .database
            .store
            .workspace(second)
            .unwrap()
            .unwrap()
            .name,
        "Other customer"
    );

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
        assert_eq!(
            fixture
                .request(
                    Some(&pending),
                    "POST",
                    &fixture.path("/overflow/wallets"),
                    Some(json!({"api_key":"fixture-key"})),
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
    let wrong_origin = format!("{}.attacker.example", fixture.state.config.public_origin);
    for (method, suffix, body) in [
        ("PUT", "/settings", json!({"name":"Intruder"})),
        (
            "POST",
            "/overflow/wallets",
            json!({"api_key":"fixture-key"}),
        ),
    ] {
        for (case, origin, csrf) in [
            ("missing-origin", None, Some(admin.csrf.as_str())),
            (
                "missing-csrf",
                Some(fixture.state.config.public_origin.as_str()),
                None,
            ),
            (
                "wrong-origin",
                Some(wrong_origin.as_str()),
                Some(admin.csrf.as_str()),
            ),
            (
                "other-sign-in-csrf",
                Some(fixture.state.config.public_origin.as_str()),
                Some(pending.csrf.as_str()),
            ),
        ] {
            let mut request = Request::builder()
                .method(method)
                .uri(fixture.path(suffix))
                .header(header::COOKIE, format!("__Host-ciproxy={}", admin.cookie))
                .header(header::CONTENT_TYPE, "application/json");
            if let Some(origin) = origin {
                request = request.header(header::ORIGIN, origin);
            }
            if let Some(csrf) = csrf {
                request = request.header("x-ciproxy-csrf", csrf);
            }
            let response = router(fixture.state.clone())
                .oneshot(request.body(Body::from(body.to_string())).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{case}");
            assert_eq!(json(response).await["code"], "access_denied", "{case}");
        }
    }
    assert_eq!(
        fixture
            .database
            .store
            .workspace(fixture.workspace)
            .unwrap()
            .unwrap()
            .name,
        "Customer"
    );

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
