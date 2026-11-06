//! A completed take-out cannot bypass the next installation's atomic transition.

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use super::fixture::Fixture;

#[tokio::test]
async fn a_completed_takeout_is_fenced_from_a_later_installation() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let mut other = fixture.organization.clone();
    other.id = Uuid::new_v4();
    other.github_id = 200;
    other.installation_id = 201;
    other.login = "other".to_owned();
    fixture.database.store.install_organization(&other).unwrap();
    fixture.remote.organizations.lock().unwrap().push(other);
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
    fixture.advance(Duration::hours(1));
    let path = fixture.path(&format!("/organizations/{}", fixture.organization.id));
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &path, None)
            .await
            .status(),
        StatusCode::OK
    );
    let mut replacement = fixture.organization.clone();
    replacement.installation_id = 1001;
    fixture
        .database
        .store
        .install_organization(&replacement)
        .unwrap();
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &path, None)
            .await
            .status(),
        StatusCode::OK
    );
    let saved = fixture
        .database
        .store
        .organization(fixture.workspace, replacement.id)
        .unwrap()
        .unwrap();
    assert_eq!(saved.app_state, AppState::Removed);
    assert!(saved.delete_after.is_none());
    assert!(
        fixture
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event == "lifecycle:1001")
    );
    assert!(
        fixture
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event == "uninstall:1001")
    );
    assert!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .is_empty()
    );
}
