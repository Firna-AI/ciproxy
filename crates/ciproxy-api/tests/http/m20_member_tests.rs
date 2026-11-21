//! Profile and access-request provenance through real OAuth persistence and HTTP.

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::workspaces::Workspace;
use diesel::prelude::*;
use serde_json::{Value, json};
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn m20_member_profiles_keep_request_scope_and_real_grant_dates() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let foreign = Uuid::new_v4();
    fixture
        .database
        .store
        .create_workspace(&Workspace {
            id: foreign,
            name: "Other workspace".into(),
            delete_after: None,
        })
        .unwrap();
    let mut organization = fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.workspace_id = foreign;
    organization.github_id = 200;
    organization.installation_id = 201;
    organization.login = "other-workspace".into();
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
                github_id: 200,
                owner: false,
            },
        ],
    );
    let requester = fixture.login(8, None).await;
    let path = fixture.path("/members");
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    let member = find(&body, "8");
    assert_eq!(member["name"], "Person 8");
    assert_eq!(member["avatar_url"], "https://github.example/avatar");
    assert_eq!(
        member["request_organizations"],
        json!([{"id":fixture.organization.id.to_string(), "login":"customer"}])
    );
    assert_eq!(member["requested_at"], "2026-11-06T12:00:00Z");
    assert_eq!(member["granted_at"], Value::Null);
    fixture.advance(Duration::hours(2));
    let member_path = fixture.path("/members/8");
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &member_path,
                Some(json!({"role":"viewer"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(find(&body, "8")["granted_at"], "2026-11-06T14:00:00Z");
    assert_eq!(find(&body, "8")["requested_at"], "2026-11-06T12:00:00Z");
    fixture.advance(Duration::hours(1));
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &member_path,
                Some(json!({"role":"viewer"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(find(&body, "8")["granted_at"], "2026-11-06T14:00:00Z");
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &member_path,
                Some(json!({"role":"billing"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(find(&body, "8")["granted_at"], "2026-11-06T15:00:00Z");
    let mut renamed = fixture.organization.clone();
    renamed.login = "renamed-customer".into();
    fixture
        .database
        .store
        .install_organization(&renamed)
        .unwrap();
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(
        find(&body, "8")["request_organizations"][0]["login"],
        "customer"
    );
    let mut connection = PgConnection::establish(&fixture.database.url).unwrap();
    diesel::sql_query("UPDATE dashboard_roles SET granted_at=NULL, request_organizations=NULL WHERE workspace_id=$1 AND github_user_id=8")
        .bind::<diesel::sql_types::Uuid,_>(fixture.workspace).execute(&mut connection).unwrap();
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(find(&body, "8")["granted_at"], Value::Null);
    assert_eq!(find(&body, "8")["request_organizations"], Value::Null);
    for suffix in ["/members", "/organizations", "/labels", "/settings"] {
        assert_eq!(
            fixture
                .request(Some(&requester), "GET", &fixture.path(suffix), None)
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            fixture
                .request(
                    Some(&admin),
                    "GET",
                    &format!("/workspaces/{foreign}{suffix}"),
                    None
                )
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &member_path, None)
            .await
            .status(),
        StatusCode::OK
    );
    let body = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(
        body["members"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|member| member["id"] == "8")
            .count(),
        0
    );
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_member_profile(8)
            .unwrap()
            .unwrap()
            .name
            .as_deref(),
        Some("Person 8")
    );
    fixture
        .database
        .store
        .save_dashboard_role(foreign, 8, None, "2026-11-06T15:00:00Z".parse().unwrap())
        .unwrap();
    assert!(
        fixture
            .database
            .store
            .dashboard_member_profile(8)
            .unwrap()
            .is_none()
    );
}

fn find<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["id"] == id)
        .unwrap()
}
