//! An empty in-flight poll releases retired binding cleanup authority.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_github::listener::{DurableListener, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use tokio::sync::watch;
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::label_retirement_support::Queue;
use super::label_retirement_tests::retire;

#[tokio::test]
async fn empty_in_flight_poll_releases_a_retired_listener_before_another_poll() {
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
    let target = ListenerTarget {
        workspace_id: fixture.workspace,
        organization_id: fixture.organization.id,
        scale_set_id: set.id,
    };
    let queue = Queue::default();
    queue.empty.store(true, Ordering::SeqCst);
    let listener = DurableListener {
        store: Arc::new(StoreAdapter::new(
            fixture.database.store.clone(),
            fixture.runtime.clone(),
        )),
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
                    owner: "idle-retirement".to_owned(),
                    capacity: 2,
                    max_polls: Some(2),
                },
                signal,
            )
            .await
    });
    queue.entered.notified().await;
    retire(&fixture, &browser, label).await;
    assert!(
        !fixture
            .remote
            .events
            .lock()
            .unwrap()
            .contains(&format!("delete:{}", set.github_id))
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
    let result = running.await.unwrap();
    assert!(matches!(result, Ok(())));
    assert_eq!(*queue.capacities.lock().unwrap(), vec![2]);
    assert!(queue.closed.load(Ordering::SeqCst));
    Maintenance {
        state: fixture.state.clone(),
    }
    .reconcile()
    .await
    .unwrap();
    assert!(
        fixture
            .remote
            .events
            .lock()
            .unwrap()
            .contains(&format!("delete:{}", set.github_id))
    );
    assert!(
        !fixture
            .remote
            .events
            .lock()
            .unwrap()
            .contains(&format!("delete:{}", replacement_set.github_id))
    );
}
