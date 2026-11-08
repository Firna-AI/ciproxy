//! Signed requester association, proof ordering, and explicit workspace selection.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use uuid::Uuid;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{
    self, begin, claims, event, other_workspace, pending, returned, scenario,
};

#[tokio::test]
async fn installation_orders_preserve_original_requester_and_never_guess_a_workspace() {
    for (callback_first, owner_first) in [(false, false), (true, false), (false, true)] {
        let (fixture, server, admin, added) = scenario().await;
        let state = begin(&fixture, &admin).await;
        if owner_first {
            server.control.person.store(8, Ordering::SeqCst);
            let owner = fixture.login(8, Some(Role::Admin)).await;
            let owner_state = link_support::start(&fixture, &owner).await;
            returned(&fixture, &owner_state, CallbackOutcome::Accepted)
                .await
                .unwrap();
            server.control.person.store(7, Ordering::SeqCst);
        }
        let provisional = fixture.database.store.claim_for_installation(1066).unwrap();
        let provisional = provisional.map(|claim| {
            (
                claim.requester_id,
                claim.workspace_id,
                fixture
                    .database
                    .store
                    .installation_workspace_offers(claim.id, 8)
                    .unwrap()
                    .is_empty(),
            )
        });
        assert_eq!(provisional, owner_first.then_some((8, None, true)));
        if callback_first {
            returned(&fixture, &state, CallbackOutcome::Accepted)
                .await
                .unwrap();
        }
        assert!(
            fixture
                .database
                .store
                .organization_by_installation_id(1066)
                .unwrap()
                .is_none()
        );
        let id = pending(
            fixture
                .app
                .installation(event(&added, 7, InstallationAction::Created))
                .await
                .unwrap(),
        );
        assert!(
            fixture
                .database
                .store
                .organization_by_installation_id(1066)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            fixture
                .database
                .store
                .installation_claim(id)
                .unwrap()
                .unwrap()
                .requester_id,
            7
        );
        let associated = fixture
            .database
            .store
            .installation_claim(id)
            .unwrap()
            .unwrap();
        assert_eq!(associated.user_verified, callback_first);
        assert!(!associated.direct);
        assert!(associated.workspace_id.is_none());
        let current = returned(&fixture, &state, CallbackOutcome::Accepted)
            .await
            .unwrap();
        let links = claims(&fixture, &current).await;
        assert_eq!(links[0]["workspace_id"], fixture.workspace.to_string());
        assert_eq!(links[0]["state"], "linked_inactive");
        assert_eq!(
            links[0]["activation"]["reasons"],
            json!(["billing_upgrade"])
        );
        let saved = fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .unwrap();
        assert_eq!(saved.workspace_id, fixture.workspace);
        assert!(
            !fixture
                .database
                .store
                .organization_runnable(fixture.workspace, saved.id)
                .unwrap()
        );
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_role(fixture.workspace, 7)
                .unwrap(),
            Some(Role::Admin)
        );
    }
}

#[tokio::test]
async fn several_admin_workspaces_require_an_exact_offered_choice_and_fresh_installation_proof() {
    let (fixture, server, admin, added) = scenario().await;
    let other = other_workspace(&fixture);
    let state = begin(&fixture, &admin).await;
    let second_attempt = begin(&fixture, &admin).await;
    assert!(state != second_attempt);
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
    let links = claims(&fixture, &current).await;
    assert_eq!(links[0]["state"], "choose_workspace");
    assert_eq!(links[0]["workspaces"].as_array().unwrap().len(), 2);
    assert!(links[0]["workspace_id"].is_null());
    let path = format!("/installation-links/{id}/workspace");
    let refused = fixture
        .request(
            Some(&current),
            "PUT",
            &path,
            Some(json!({"workspace_id":Uuid::new_v4()})),
        )
        .await;
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);
    server
        .control
        .fail_user_installations
        .store(true, Ordering::SeqCst);
    let refused = fixture
        .request(
            Some(&current),
            "PUT",
            &path,
            Some(json!({"workspace_id":other.id})),
        )
        .await;
    assert_eq!(refused.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .is_none()
    );
    server
        .control
        .fail_user_installations
        .store(false, Ordering::SeqCst);
    server
        .control
        .hide_added_installation
        .store(true, Ordering::SeqCst);
    let refused = fixture
        .request(
            Some(&current),
            "PUT",
            &path,
            Some(json!({"workspace_id":other.id})),
        )
        .await;
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);
    assert!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .is_none()
    );
    server
        .control
        .hide_added_installation
        .store(false, Ordering::SeqCst);
    let selected = fixture
        .request(
            Some(&current),
            "PUT",
            &path,
            Some(json!({"workspace_id":other.id})),
        )
        .await;
    assert_eq!(selected.status(), StatusCode::OK);
    assert_eq!(json(selected).await["workspace_id"], other.id.to_string());
    let replay = fixture
        .request(
            Some(&current),
            "PUT",
            &path,
            Some(json!({"workspace_id":fixture.workspace})),
        )
        .await;
    assert_eq!(replay.status(), StatusCode::CONFLICT);
    assert_eq!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .unwrap()
            .workspace_id,
        other.id
    );
}
