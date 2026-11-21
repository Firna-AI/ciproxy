//! The signed install requester becomes the first Admin through the real App service.

use axum::http::StatusCode;
use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::{
    Account, AccountKind, Actor, Installation, InstallationAction, InstallationEvent,
};
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::organizations::RepositoryAccess;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn signed_install_requester_gets_admin_once_and_owner_does_not_replace_them() {
    let fixture = Fixture::new();
    let mut organization = fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.github_id = 200;
    organization.installation_id = 201;
    organization.login = "new-customer".to_owned();
    organization.repositories = RepositoryAccess::AllPrivate;
    fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(organization);
    let event = InstallationEvent {
        action: InstallationAction::Created,
        installation: Installation {
            id: 201,
            account: Account {
                id: 200,
                login: "new-customer".to_owned(),
                kind: AccountKind::Organization,
            },
        },
        requester: Some(Actor { id: 77 }),
        sender: Actor { id: 88 },
    };
    let outcome = fixture.app.installation(event.clone()).await.unwrap();
    let InstallationOutcome::Organization(installed) = outcome else {
        panic!("expected organization")
    };
    assert_ne!(installed.workspace_id, fixture.workspace);
    fixture.identity.memberships.lock().unwrap().insert(
        77,
        vec![Membership {
            github_id: 200,
            owner: false,
        }],
    );
    let browser = fixture.login(77, None).await;
    let session = json(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(session["workspaces"][0]["role"], "admin");
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(installed.workspace_id, 77)
            .unwrap(),
        Some(Role::Admin)
    );
    assert!(
        fixture
            .database
            .store
            .dashboard_role(installed.workspace_id, 88)
            .unwrap()
            .is_none()
    );
    fixture
        .database
        .store
        .save_dashboard_role(
            installed.workspace_id,
            77,
            Some(Role::Viewer),
            "2026-11-06T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    fixture.app.installation(event).await.unwrap();
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(installed.workspace_id, 77)
            .unwrap(),
        Some(Role::Viewer)
    );
    let personal = InstallationEvent {
        action: InstallationAction::Created,
        installation: Installation {
            id: 901,
            account: Account {
                id: 77,
                login: "person-77".to_owned(),
                kind: AccountKind::User,
            },
        },
        requester: None,
        sender: Actor { id: 77 },
    };
    assert!(matches!(
        fixture.app.installation(personal).await.unwrap(),
        InstallationOutcome::OrganizationsOnly { .. }
    ));
    assert_eq!(
        fixture
            .request(Some(&browser), "GET", "/session", None)
            .await
            .status(),
        StatusCode::OK
    );
}
