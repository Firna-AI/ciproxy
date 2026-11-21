//! A saved take-out succeeds while expected runtime work remains pending.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::organizations::AppState;
use serde_json::json;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn pending_takeout_returns_success_without_remote_finish_and_keeps_unexpected_errors() {
    let fixture = Fixture::new();
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
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let path = fixture.path(&format!("/organizations/{}", fixture.organization.id));
    fixture
        .remote
        .pending_lifecycle
        .store(true, Ordering::SeqCst);
    let response = fixture.request(Some(&admin), "DELETE", &path, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await, json!({"ok": true}));
    assert_eq!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fixture
            .database
            .store
            .pending_github_lifecycle(100)
            .unwrap()
            .len(),
        1
    );
    let saved = fixture
        .database
        .store
        .organization(fixture.workspace, fixture.organization.id)
        .unwrap()
        .unwrap();
    assert_eq!(saved.app_state, AppState::Removed);
    assert_eq!(saved.removed_at, None);
    assert_eq!(saved.delete_after, None);
    assert_eq!(
        fixture
            .database
            .store
            .scale_sets(fixture.workspace, fixture.organization.id)
            .unwrap()
            .len(),
        1
    );
    assert!(
        !fixture
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.starts_with("delete:") || event.starts_with("uninstall:"))
    );
    fixture
        .remote
        .pending_lifecycle
        .store(false, Ordering::SeqCst);
    fixture.remote.fail_lifecycle.store(true, Ordering::SeqCst);
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &path, None)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .len(),
        1
    );
    assert!(
        !fixture
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.starts_with("delete:") || event.starts_with("uninstall:"))
    );
    Maintenance {
        state: fixture.state.clone(),
    }
    .reconcile()
    .await
    .unwrap();
    assert_eq!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        fixture
            .database
            .store
            .pending_github_lifecycle(100)
            .unwrap()
            .len(),
        0
    );
    assert!(
        fixture
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event == "uninstall:101")
    );
}
