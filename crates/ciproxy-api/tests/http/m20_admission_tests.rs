//! Settings separates the Admin add capability from a new admission hold.

use axum::http::StatusCode;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::link_support::tier;

#[tokio::test]
async fn m20_settings_predicts_new_organization_hold_without_changing_add_permission() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let path = fixture.path("/settings");
    let free = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(free["can_add_organization"], true);
    assert_eq!(free["new_organization_requires_upgrade"], true);
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/organizations/add"),
                None
            )
            .await
            .status(),
        StatusCode::OK
    );
    tier(&fixture, fixture.workspace, Tier::Team);
    let team = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(team["can_add_organization"], true);
    assert_eq!(team["new_organization_requires_upgrade"], false);
    tier(&fixture, fixture.workspace, Tier::Free);
    let mut removed = fixture.organization.clone();
    removed.id = Uuid::new_v4();
    removed.github_id = 200;
    removed.installation_id = 201;
    removed.login = "removed-history".into();
    fixture
        .database
        .store
        .install_organization(&removed)
        .unwrap();
    let at = "2026-11-06T12:00:00Z".parse().unwrap();
    fixture
        .database
        .store
        .record_app_removal(fixture.workspace, removed.id, at)
        .unwrap();
    let mixed = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(mixed["organizations"].as_array().unwrap().len(), 2);
    assert_eq!(mixed["can_add_organization"], true);
    assert_eq!(mixed["new_organization_requires_upgrade"], true);
    fixture
        .database
        .store
        .record_app_removal(fixture.workspace, fixture.organization.id, at)
        .unwrap();
    let records = fixture
        .database
        .store
        .organizations(fixture.workspace)
        .unwrap();
    assert_eq!(
        records
            .iter()
            .filter(|record| record.app_state != AppState::Removed)
            .count(),
        0
    );
    assert_eq!(
        fixture
            .request(Some(&admin), "GET", &path, None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
}
