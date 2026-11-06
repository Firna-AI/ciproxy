//! Role loss before link does not grant Admin, and retained holds cannot be bypassed.

use axum::http::StatusCode;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{
    begin, claims, event, maintenance, memberships, other_workspace, pending, returned, scenario,
    second_admin, tier,
};

#[tokio::test]
async fn exact_prior_offer_survives_role_loss_without_resurrecting_admin_and_holds_survive_reinstall()
 {
    for keep_other_role in [true, false] {
        let (fixture, server, admin, mut added) = scenario().await;
        let _other = other_workspace(&fixture);
        let state = begin(&fixture, &admin).await;
        let id = pending(
            fixture
                .app
                .installation(event(&added, 7, InstallationAction::Created))
                .await
                .unwrap(),
        );
        let current = returned(&fixture, &state, CallbackOutcome::Accepted)
            .await
            .unwrap();
        assert_eq!(
            claims(&fixture, &current).await[0]["state"],
            "choose_workspace"
        );
        let memberships_now: &[i64] = if keep_other_role { &[106, 200] } else { &[106] };
        memberships(&server, memberships_now);
        let selected = fixture
            .request(
                Some(&current),
                "PUT",
                &format!("/installation-links/{id}/workspace"),
                Some(json!({"workspace_id":fixture.workspace})),
            )
            .await;
        assert_eq!(selected.status(), StatusCode::OK);
        let view = json(selected).await;
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_user_by_github(7)
                .unwrap()
                .is_some(),
            keep_other_role
        );
        assert_eq!(view["workspace_id"], fixture.workspace.to_string());
        assert_eq!(
            view["activation"]["reasons"],
            json!(["billing_upgrade", "admin_approval"])
        );
        assert_eq!(view["activation"]["can_enable"], false);
        assert!(view["activation"]["upgrade"].is_null());
        assert_eq!(view["activation"]["ask_admin"], true);
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_role(fixture.workspace, 7)
                .unwrap(),
            None
        );
        let saved = fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .unwrap();
        for path in [
            fixture.path("/settings"),
            fixture.path(&format!("/organizations/{}/enable", saved.id)),
        ] {
            let method = if path.ends_with("enable") {
                "POST"
            } else {
                "GET"
            };
            assert_eq!(
                fixture
                    .request(Some(&current), method, &path, None)
                    .await
                    .status(),
                StatusCode::FORBIDDEN
            );
        }
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
        let retained = fixture
            .database
            .store
            .organization_activation(fixture.workspace, saved.id)
            .unwrap();
        assert!(!retained.admitted);
        assert!(retained.billing_hold && retained.admin_approval_hold);
        tier(&fixture, fixture.workspace, Tier::Team);
        maintenance(&fixture).await;
        let held = fixture
            .database
            .store
            .organization_activation(fixture.workspace, saved.id)
            .unwrap();
        assert!(!held.billing_hold && held.admin_approval_hold && !held.admitted);
        let next_admin = second_admin(&fixture, &server).await;
        let enabled = fixture
            .request(
                Some(&next_admin),
                "POST",
                &fixture.path(&format!("/organizations/{}/enable", saved.id)),
                None,
            )
            .await;
        assert_eq!(enabled.status(), StatusCode::OK);
        assert!(
            fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
        assert_ne!(
            fixture
                .database
                .store
                .dashboard_role(fixture.workspace, 7)
                .unwrap(),
            Some(Role::Admin)
        );
    }
}
