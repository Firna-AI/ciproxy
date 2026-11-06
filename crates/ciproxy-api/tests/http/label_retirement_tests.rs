//! Label deletion keeps completion tracking through polls, re-create, and restart.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_api::runtime::Runtime;
use ciproxy_github::error::Error;
use ciproxy_github::listener::{DurableListener, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::persistence::session::{ListenerTarget, SessionStore};
use ciproxy_github::registrations::{RegistrationService, Removal, RunnerRegistrations};
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::scale_sets::ScaleSetState;
use serde_json::json;
use tokio::sync::watch;
use uuid::Uuid;

use super::fixture::{Browser, Fixture};
use super::fixture_io::json;
use super::label_retirement_support::{Queue, started};

pub(super) async fn retire(fixture: &Fixture, browser: &Browser, label: Uuid) {
    let response = fixture
        .request(
            Some(browser),
            "DELETE",
            &fixture.path(&format!("/labels/{label}")),
            None,
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}

async fn keeps_completion(restarted: bool) {
    let fixture = Fixture::new();
    let browser = fixture.login(7, Some(Role::Admin)).await;
    let input = json!({"name":"custom","vcpus":2,"memory_gb":8,"disk_gb":20,"allow_machines":true,"allow_boat":true,"when_nothing_fits":"wait"});
    let created = fixture
        .request(
            Some(&browser),
            "POST",
            &fixture.path("/labels"),
            Some(input.clone()),
        )
        .await;
    assert_eq!(created.status(), StatusCode::OK);
    let label: Uuid = json(created).await["id"].as_str().unwrap().parse().unwrap();
    let set = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == label)
        .unwrap();
    fixture
        .database
        .store
        .save_github_job(fixture.workspace, &started(&set), fixture.runtime.now())
        .unwrap();
    if restarted {
        retire(&fixture, &browser, label).await;
    }
    let adapter = Arc::new(StoreAdapter::new(
        fixture.database.store.clone(),
        fixture.runtime.clone(),
    ));
    let target = ListenerTarget {
        workspace_id: fixture.workspace,
        organization_id: fixture.organization.id,
        scale_set_id: set.id,
    };
    let queue = Queue::default();
    let listener = DurableListener {
        store: adapter.clone(),
        rest: fixture.remote.clone(),
        scale_sets: Arc::new(queue.clone()),
        output: Arc::new(queue.clone()),
    };
    let (_stop, signal) = watch::channel(false);
    let running = tokio::spawn(async move {
        listener
            .run(
                target,
                ListenerOptions {
                    owner: "retirement".to_owned(),
                    capacity: 2,
                    max_polls: Some(1),
                },
                signal,
            )
            .await
    });
    if !restarted {
        queue.entered.notified().await;
        retire(&fixture, &browser, label).await;
    }
    let context = adapter
        .context(target)
        .await
        .ok()
        .map(|context| context.scale_set.id);
    assert_eq!(
        context,
        Some(set.id),
        "retired binding must keep exact completion context"
    );
    let discovered = fixture
        .database
        .store
        .listener_scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap();
    assert!(
        discovered
            .iter()
            .any(|binding| binding.scale_set.id == set.id
                && binding.state == ScaleSetState::Draining
                && binding.has_tracked_jobs)
    );
    let registrations = RegistrationService {
        store: adapter.clone(),
        scale_sets: fixture.state.scale_sets.clone(),
    };
    assert!(matches!(
        registrations.create(target, "new-runner").await,
        Err(Error::OrganizationInactive)
    ));
    assert_eq!(
        registrations.remove(target, 15).await.unwrap(),
        Removal::Removed
    );
    assert!(
        !fixture
            .remote
            .events
            .lock()
            .unwrap()
            .contains(&format!("delete:{}", set.github_id)),
        "remote deletion must wait for the running job"
    );
    let replacement = fixture
        .request(
            Some(&browser),
            "POST",
            &fixture.path("/labels"),
            Some(input),
        )
        .await;
    assert_eq!(replacement.status(), StatusCode::OK);
    let replacement: Uuid = json(replacement).await["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let replacement_set = fixture
        .database
        .store
        .scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == replacement)
        .unwrap();
    assert_ne!(replacement_set.github_id, set.github_id);
    queue.release.notify_one();
    let outcome = running.await.unwrap();
    assert!(
        matches!(outcome, Ok(())),
        "retired listener must save and acknowledge its final batch"
    );
    let saved = fixture
        .database
        .store
        .github_jobs(fixture.workspace, fixture.organization.id)
        .unwrap();
    let completed = saved
        .iter()
        .find(|job| job.id == Uuid::from_u128(0x71))
        .unwrap();
    assert_eq!(completed.phase, DemandPhase::Completed);
    assert_eq!(completed.result.as_deref(), Some("succeeded"));
    assert_eq!(
        completed.completed_at,
        Some("2026-11-06T12:00:01Z".parse().unwrap())
    );
    assert_eq!(completed.label_id, label);
    assert_eq!(completed.scale_set_id, set.id);
    assert_eq!(*queue.acquired.lock().unwrap(), Vec::<i64>::new());
    assert_eq!(*queue.acknowledged.lock().unwrap(), vec![7]);
    assert!(queue.closed.load(Ordering::SeqCst));
    let expected_capacity = if restarted { 0 } else { 2 };
    assert_eq!(*queue.capacities.lock().unwrap(), vec![expected_capacity]);
    assert_eq!(adapter.context(target).await.unwrap().scale_set.id, set.id);
    Maintenance {
        state: fixture.state.clone(),
    }
    .reconcile()
    .await
    .unwrap();
    Maintenance {
        state: fixture.state.clone(),
    }
    .reconcile()
    .await
    .unwrap();
    let discovered = fixture
        .database
        .store
        .listener_scale_sets(fixture.workspace, fixture.organization.id)
        .unwrap();
    assert!(
        !discovered
            .iter()
            .any(|binding| binding.scale_set.id == set.id)
    );
    assert!(
        discovered
            .iter()
            .any(|binding| binding.scale_set.id == replacement_set.id
                && binding.state == ScaleSetState::Active)
    );
    assert_eq!(
        fixture
            .database
            .store
            .scale_set_binding(fixture.workspace, fixture.organization.id, set.id)
            .unwrap()
            .unwrap()
            .state,
        ScaleSetState::Removed
    );
    let events = fixture.remote.events.lock().unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == format!("delete:{}", set.github_id))
            .count(),
        1
    );
    assert!(!events.contains(&format!("delete:{}", replacement_set.github_id)));
}

#[tokio::test]
async fn retirement_during_poll_saves_completion_without_acquiring_new_work() {
    keeps_completion(false).await;
}

#[tokio::test]
async fn retired_binding_resumes_after_restart_until_completion() {
    keeps_completion(true).await;
}
