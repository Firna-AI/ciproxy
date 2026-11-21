//! Independent billing admission, linked counts, and existing active reinstall behavior.

use axum::http::StatusCode;
use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use uuid::Uuid;

use super::fixture_io::CallbackOutcome;
use super::link_guard_support::refuses_runners;
use super::link_support::{
    begin, claims, event, maintenance, return_install, returned, scenario, tier,
};

#[tokio::test]
async fn free_admission_waits_for_upgrade_or_room_and_downgrade_keeps_admitted_organizations() {
    for upgrade in [true, false] {
        let (fixture, server, admin, mut added) = scenario().await;
        let state = begin(&fixture, &admin).await;
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap();
        let current = returned(&fixture, &state, CallbackOutcome::Accepted)
            .await
            .unwrap();
        let saved = fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .unwrap();
        assert!(
            !fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
        assert!(
            fixture
                .database
                .store
                .organization_runnable(fixture.workspace, fixture.organization.id)
                .unwrap()
        );
        assert!(
            fixture
                .database
                .store
                .scale_sets(fixture.workspace, saved.id)
                .unwrap()
                .is_empty()
        );
        refuses_runners(&fixture, saved.id).await;
        let links = claims(&fixture, &current).await;
        assert_eq!(
            links[0]["activation"]["upgrade"]["workspace_id"],
            fixture.workspace.to_string()
        );
        let mut extra = added.clone();
        extra.id = Uuid::new_v4();
        extra.github_id = 107;
        extra.installation_id = 1068;
        extra.login = "another-org".to_owned();
        fixture
            .remote
            .organizations
            .lock()
            .unwrap()
            .push(extra.clone());
        server
            .control
            .extra_installations
            .lock()
            .unwrap()
            .push((1068, 107, extra.login.clone()));
        let next = begin(&fixture, &current).await;
        fixture
            .app
            .installation(event(&extra, 7, InstallationAction::Created))
            .await
            .unwrap();
        return_install(&fixture, &next, 1068, CallbackOutcome::Accepted)
            .await
            .unwrap();
        let extra_saved = fixture
            .database
            .store
            .organization_by_installation_id(1068)
            .unwrap()
            .unwrap();
        assert!(
            !fixture
                .database
                .store
                .organization_runnable(fixture.workspace, extra_saved.id)
                .unwrap()
        );
        let unblock_status = if upgrade {
            fixture
                .database
                .store
                .save_dashboard_role(fixture.workspace, 7, None, fixture.runtime.now())
                .unwrap();
            tier(&fixture, fixture.workspace, Tier::Team);
            StatusCode::OK
        } else {
            fixture
                .request(
                    Some(&current),
                    "DELETE",
                    &fixture.path(&format!("/organizations/{}", fixture.organization.id)),
                    None,
                )
                .await
                .status()
        };
        assert_eq!(unblock_status, StatusCode::OK, "upgrade={upgrade}");
        let expected_requester_role = if upgrade { None } else { Some(Role::Admin) };
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_role(fixture.workspace, 7)
                .unwrap(),
            expected_requester_role
        );
        maintenance(&fixture).await;
        assert_eq!(
            fixture
                .database
                .store
                .organization_activation(fixture.workspace, saved.id)
                .unwrap()
                .admitted,
            upgrade
        );
        assert_eq!(
            fixture
                .database
                .store
                .organization_activation(fixture.workspace, extra_saved.id)
                .unwrap()
                .admitted,
            upgrade
        );
        let room_status = if upgrade {
            StatusCode::OK
        } else {
            fixture
                .request(
                    Some(&current),
                    "DELETE",
                    &fixture.path(&format!("/organizations/{}", extra_saved.id)),
                    None,
                )
                .await
                .status()
        };
        assert_eq!(room_status, StatusCode::OK);
        maintenance(&fixture).await;
        assert!(
            fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
        let active_events=fixture.remote.lifecycle_events.lock().unwrap().iter().filter(|event|matches!(event,LifecycleEvent::Active(organization) if organization.id==saved.id)).count();
        assert_eq!(
            active_events, 1,
            "admission must publish Active even when GitHub AppState stays Active"
        );
        tier(&fixture, fixture.workspace, Tier::Free);
        maintenance(&fixture).await;
        assert!(
            fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Deleted))
            .await
            .unwrap();
        added.installation_id = 1067;
        fixture
            .remote
            .organizations
            .lock()
            .unwrap()
            .iter_mut()
            .find(|organization| organization.github_id == 106)
            .unwrap()
            .installation_id = 1067;
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap();
        assert!(
            fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
    }
}
