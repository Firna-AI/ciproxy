//! Retained admission prepares current bindings before Add publishes activation.

use axum::http::StatusCode;
use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use uuid::Uuid;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{
    begin, event, other_workspace, return_install, returned, scenario, tier,
};

#[tokio::test]
async fn retained_add_reinstall_saves_new_label_bindings_before_active_even_after_admin_loss() {
    for lose_admin in [false, true] {
        let (fixture, server, admin, mut added) = scenario().await;
        tier(&fixture, fixture.workspace, Tier::Team);
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
            fixture
                .database
                .store
                .organization_activation(fixture.workspace, saved.id)
                .unwrap()
                .admitted
        );
        tier(&fixture, fixture.workspace, Tier::Free);
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Deleted))
            .await
            .unwrap();
        let created = fixture
            .request(
                Some(&current),
                "POST",
                &fixture.path("/labels"),
                Some(json!({
                    "name": "during-removal", "vcpus": 2, "memory_gb": 8,
                    "disk_gb": 20, "allow_machines": true, "allow_boat": true,
                    "when_nothing_fits": "wait"
                })),
            )
            .await;
        assert_eq!(created.status(), StatusCode::OK);
        let label: Uuid = json(created).await["id"].as_str().unwrap().parse().unwrap();
        assert!(
            !fixture
                .database
                .store
                .scale_sets(fixture.workspace, saved.id)
                .unwrap()
                .iter()
                .any(|set| set.label_id == label)
        );

        let _other = other_workspace(&fixture);
        let state = begin(&fixture, &current).await;
        added.installation_id = 1067;
        fixture
            .remote
            .organizations
            .lock()
            .unwrap()
            .iter_mut()
            .find(|organization| organization.github_id == added.github_id)
            .unwrap()
            .installation_id = 1067;
        server.control.extra_installations.lock().unwrap().push((
            1067,
            added.github_id,
            added.login.clone(),
        ));
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap();
        let continuing = return_install(&fixture, &state, 1067, CallbackOutcome::Accepted)
            .await
            .unwrap();
        let claim = fixture
            .database
            .store
            .claim_for_installation(1067)
            .unwrap()
            .unwrap();
        assert_eq!(claim.organization_id, None);
        if lose_admin {
            fixture
                .database
                .store
                .save_dashboard_role(
                    fixture.workspace,
                    7,
                    Some(Role::Viewer),
                    fixture.runtime.now(),
                )
                .unwrap();
        }
        let response = fixture
            .request(
                Some(&continuing),
                "PUT",
                &format!("/installation-links/{}/workspace", claim.id),
                Some(json!({"workspace_id":fixture.workspace})),
            )
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json(response).await["state"], "linked");
        let bindings = fixture.remote.active_bindings.lock().unwrap();
        assert!(
            bindings.get(&1067).unwrap().contains(&label),
            "lose_admin={lose_admin}: current bindings must exist when Active is published"
        );
        assert!(
            fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
    }
}
