//! One failed lifecycle delivery cannot block another organization's cleanup.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::error::Error;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::fixture::Fixture;

#[tokio::test]
async fn lifecycle_failure_keeps_its_row_and_allows_other_takeout_cleanup() {
    let fixture = Fixture::new();
    let second = Uuid::new_v4();
    fixture
        .database
        .store
        .create_workspace(&Workspace {
            id: second,
            name: "Second workspace".to_owned(),
            delete_after: None,
        })
        .unwrap();
    let mut targets = vec![(fixture.workspace, fixture.organization.id)];
    for (workspace, github_id, login) in [
        (fixture.workspace, 200, "first-stays"),
        (second, 300, "second-target"),
        (second, 400, "second-stays"),
    ] {
        let mut organization = fixture.organization.clone();
        organization.id = Uuid::new_v4();
        organization.workspace_id = workspace;
        organization.github_id = github_id;
        organization.installation_id = github_id + 1;
        organization.login = login.to_owned();
        fixture
            .database
            .store
            .install_organization(&organization)
            .unwrap();
        fixture
            .remote
            .organizations
            .lock()
            .unwrap()
            .push(organization.clone());
        if github_id == 300 {
            targets.push((workspace, organization.id));
        }
    }
    fixture.identity.memberships.lock().unwrap().insert(
        7,
        [100, 200, 300, 400]
            .into_iter()
            .map(|github_id| Membership {
                github_id,
                owner: false,
            })
            .collect(),
    );
    let browser = fixture.login(7, Some(Role::Admin)).await;
    fixture
        .database
        .store
        .save_dashboard_role(
            second,
            7,
            Some(Role::Admin),
            "2026-11-06T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    for (workspace, organization) in targets {
        fixture.remote.fail_lifecycle.store(true, Ordering::SeqCst);
        let response = fixture
            .request(
                Some(&browser),
                "DELETE",
                &format!("/workspaces/{workspace}/organizations/{organization}"),
                None,
            )
            .await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
    assert_eq!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .len(),
        2
    );
    fixture.remote.fail_lifecycle.store(true, Ordering::SeqCst);
    let maintenance = Maintenance {
        state: fixture.state.clone(),
    };
    assert!(matches!(
        maintenance.reconcile().await,
        Err(Error::Unavailable)
    ));
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
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.starts_with("uninstall:"))
            .count(),
        1
    );
    maintenance.reconcile().await.unwrap();
    assert!(
        fixture
            .database
            .store
            .pending_organization_takeouts()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fixture
            .remote
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.starts_with("uninstall:"))
            .count(),
        2
    );
}
