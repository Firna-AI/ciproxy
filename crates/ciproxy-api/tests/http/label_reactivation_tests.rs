//! Label replacements survive inactive organizations and retain exact history.

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::{
    Account, AccountKind, Actor, Installation, InstallationAction, InstallationEvent,
};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetState};
use serde_json::json;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::label_retirement_support::started;

fn event(organization: &Organization, action: InstallationAction) -> InstallationEvent {
    InstallationEvent {
        action,
        installation: Installation {
            id: organization.installation_id,
            account: Account {
                id: organization.github_id,
                login: organization.login.clone(),
                kind: AccountKind::Organization,
            },
        },
        requester: None,
        sender: Actor { id: 7 },
    }
}

async fn reactivates(action: InstallationAction) {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let mut other = fixture.organization.clone();
    other.id = Uuid::new_v4();
    other.github_id = 200;
    other.installation_id = 201;
    other.login = "other-customer".to_owned();
    fixture.database.store.install_organization(&other).unwrap();
    fixture
        .remote
        .organizations
        .lock()
        .unwrap()
        .push(other.clone());
    let default = fixture
        .database
        .store
        .labels(fixture.workspace)
        .unwrap()
        .remove(0);
    fixture
        .database
        .store
        .create_scale_set(
            fixture.workspace,
            &ScaleSet {
                id: Uuid::new_v4(),
                organization_id: other.id,
                label_id: default.id,
                github_id: 601,
                runner_group_id: 97,
            },
        )
        .unwrap();
    fixture
        .remote
        .sets
        .lock()
        .unwrap()
        .insert((default.name, 97), 601);
    let input = json!({"name":"custom","vcpus":2,"memory_gb":8,"disk_gb":20,"allow_machines":true,"allow_boat":true,"when_nothing_fits":"wait"});
    let created = fixture
        .request(
            Some(&admin),
            "POST",
            &fixture.path("/labels"),
            Some(input.clone()),
        )
        .await;
    assert_eq!(created.status(), StatusCode::OK);
    let original: Uuid = json(created).await["id"].as_str().unwrap().parse().unwrap();
    let active_set = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == original)
        .unwrap();
    let old_set = fixture
        .database
        .store
        .scale_sets(fixture.workspace, other.id)
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == original)
        .unwrap();
    assert_eq!(old_set.runner_group_id, 97);
    assert_ne!(active_set.github_id, old_set.github_id);
    let mut completed = started(&old_set);
    completed.phase = DemandPhase::Completed;
    completed.result = Some("succeeded".to_owned());
    completed.completed_at = Some("2026-11-06T12:00:01Z".parse().unwrap());
    fixture
        .database
        .store
        .save_github_job(fixture.workspace, &completed, fixture.runtime.now())
        .unwrap();
    fixture
        .database
        .store
        .confirm_github_job(fixture.workspace, other.id, completed.id, 1)
        .unwrap();
    if action == InstallationAction::Created {
        fixture
            .app
            .installation(event(&other, InstallationAction::Deleted))
            .await
            .unwrap();
    } else {
        fixture
            .database
            .store
            .set_app_suspended(fixture.workspace, other.id, true)
            .unwrap();
    }
    let deleted = fixture
        .request(
            Some(&admin),
            "DELETE",
            &fixture.path(&format!("/labels/{original}")),
            None,
        )
        .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let created = fixture
        .request(Some(&admin), "POST", &fixture.path("/labels"), Some(input))
        .await;
    assert_eq!(created.status(), StatusCode::OK);
    let replacement: Uuid = json(created).await["id"].as_str().unwrap().parse().unwrap();
    assert_ne!(replacement, original);
    let replacement_active = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == replacement)
        .unwrap();
    assert_eq!(
        fixture
            .remote
            .sets
            .lock()
            .unwrap()
            .get(&("custom".to_owned(), 97)),
        Some(&old_set.github_id)
    );
    if action == InstallationAction::Created {
        other.installation_id = 901;
        fixture
            .remote
            .organizations
            .lock()
            .unwrap()
            .iter_mut()
            .find(|saved| saved.id == other.id)
            .unwrap()
            .installation_id = other.installation_id;
    }
    let outcome = fixture.app.installation(event(&other, action)).await;
    assert!(
        matches!(outcome, Ok(InstallationOutcome::Organization(_))),
        "{action:?}: label replacement must not block activation: {outcome:?}"
    );
    let current = fixture
        .database
        .store
        .organization(fixture.workspace, other.id)
        .unwrap()
        .unwrap();
    assert_eq!(current.app_state, AppState::Active);
    assert_eq!(current.installation_id, other.installation_id);
    let replacement_set = fixture
        .database
        .store
        .scale_sets(fixture.workspace, other.id)
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == replacement)
        .unwrap();
    assert_ne!(replacement_set.github_id, old_set.github_id);
    assert_eq!(replacement_set.runner_group_id, 97);
    assert_eq!(
        fixture
            .remote
            .sets
            .lock()
            .unwrap()
            .get(&(format!("ciproxy-retired-{}", old_set.id.simple()), 97)),
        None
    );
    let retained = fixture
        .database
        .store
        .scale_set_binding(fixture.workspace, other.id, old_set.id)
        .unwrap()
        .unwrap();
    assert_eq!(retained.scale_set, old_set);
    assert_eq!(
        retained.state,
        ScaleSetState::Removed,
        "activation must complete the same idle retirement as label cleanup"
    );
    assert_eq!(
        fixture
            .database
            .store
            .github_jobs(fixture.workspace, other.id)
            .unwrap(),
        vec![completed.clone()]
    );
    let removed = fixture
        .database
        .store
        .scale_set_binding(fixture.workspace, fixture.organization.id, active_set.id)
        .unwrap()
        .unwrap();
    assert_eq!(removed.scale_set, active_set);
    assert_eq!(removed.state, ScaleSetState::Removed);
    fixture
        .app
        .refresh_repositories(other.installation_id)
        .await
        .unwrap();
    Maintenance {
        state: fixture.state.clone(),
    }
    .reconcile()
    .await
    .unwrap();
    let retained = fixture
        .database
        .store
        .scale_set_binding(fixture.workspace, other.id, old_set.id)
        .unwrap()
        .unwrap();
    assert_eq!(retained.scale_set, old_set);
    assert_eq!(retained.state, ScaleSetState::Removed);
    assert_eq!(
        fixture
            .database
            .store
            .github_jobs(fixture.workspace, other.id)
            .unwrap(),
        vec![completed]
    );
    let events = fixture.remote.events.lock().unwrap();
    assert!(events.contains(&format!("delete:{}", old_set.github_id)));
    assert!(!events.contains(&format!("delete:{}", replacement_set.github_id)));
    assert!(!events.contains(&format!("delete:{}", replacement_active.github_id)));
}

#[tokio::test]
async fn deleted_label_replacement_does_not_block_a_suspended_organization_resume() {
    reactivates(InstallationAction::Unsuspend).await;
}

#[tokio::test]
async fn deleted_label_replacement_does_not_block_a_retained_organization_reinstall() {
    reactivates(InstallationAction::Created).await;
}
