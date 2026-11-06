//! Final workspace-locked admission rechecks after remote preparation.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_api::router::router;
use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::billing::state::Tier;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{
    begin, claims, event, other_workspace, pending, return_install, returned, scenario, tier,
};
use super::ports_gate::ReadGate;

#[tokio::test]
async fn final_link_rechecks_admin_after_remote_preparation() {
    let (fixture, _server, admin, added) = scenario().await;
    other_workspace(&fixture);
    tier(&fixture, fixture.workspace, Tier::Team);
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
    let gate = Arc::new(ReadGate::default());
    *fixture.remote.ensure_gate.lock().unwrap() = Some(gate.clone());
    let state = fixture.state.clone();
    let target = fixture.workspace;
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/installation-links/{id}/workspace"))
        .header(header::COOKIE, format!("__Host-ciproxy={}", current.cookie))
        .header(header::ORIGIN, &state.config.public_origin)
        .header("x-ciproxy-csrf", &current.csrf)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"workspace_id":target}).to_string()))
        .unwrap();
    let choosing = tokio::spawn(async move { router(state).oneshot(request).await.unwrap() });
    tokio::time::timeout(Duration::from_secs(20), gate.entered.notified())
        .await
        .expect("choice did not reach preparation");
    fixture
        .database
        .store
        .save_dashboard_role(target, 7, None, fixture.runtime.now())
        .unwrap();
    gate.release.notify_one();
    let response = tokio::time::timeout(Duration::from_secs(20), choosing)
        .await
        .expect("choice did not finish")
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let data = json(response).await;
    assert_eq!(data["state"], "linked_inactive");
    assert_eq!(data["activation"]["reasons"], json!(["admin_approval"]));
    assert_eq!(data["activation"]["can_enable"], false);
    let saved = fixture
        .database
        .store
        .organization_by_installation_id(1066)
        .unwrap()
        .unwrap();
    assert_eq!(saved.workspace_id, target);
    assert!(
        !fixture
            .database
            .store
            .organization_runnable(target, saved.id)
            .unwrap()
    );
    assert!(
        fixture
            .database
            .store
            .scale_sets(target, saved.id)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fixture.database.store.dashboard_role(target, 7).unwrap(),
        None
    );
}

#[tokio::test]
async fn final_activation_rechecks_linked_count_after_remote_preparation() {
    let (fixture, server, admin, added) = scenario().await;
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
    let removal = fixture
        .request(
            Some(&current),
            "DELETE",
            &fixture.path(&format!("/organizations/{}", fixture.organization.id)),
            None,
        )
        .await;
    assert_eq!(removal.status(), StatusCode::OK);
    let gate = Arc::new(ReadGate::default());
    *fixture.remote.ensure_gate.lock().unwrap() = Some(gate.clone());
    let maintenance = Maintenance {
        state: fixture.state.clone(),
    };
    let activating = tokio::spawn(async move { maintenance.reconcile().await });
    tokio::time::timeout(Duration::from_secs(20), gate.entered.notified())
        .await
        .expect("maintenance did not reach preparation");
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
    gate.release.notify_one();
    let result = tokio::time::timeout(Duration::from_secs(20), activating)
        .await
        .expect("activation did not finish")
        .unwrap();
    assert!(matches!(result, Ok(())));
    let extra_saved = fixture
        .database
        .store
        .organization_by_installation_id(1068)
        .unwrap()
        .unwrap();
    for organization in [saved.id, extra_saved.id] {
        let local = fixture
            .database
            .store
            .organization_activation(fixture.workspace, organization)
            .unwrap();
        assert!(
            !local.admitted,
            "organization {organization} must wait while two linked organizations remain on Free"
        );
        assert!(local.billing_hold);
        assert!(
            !fixture
                .database
                .store
                .organization_runnable(fixture.workspace, organization)
                .unwrap()
        );
    }
}
