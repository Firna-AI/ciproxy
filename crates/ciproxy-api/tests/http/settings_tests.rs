//! Settings, labels, repository isolation, and intentional-removal recovery.

use axum::http::StatusCode;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use std::sync::atomic::Ordering;

use super::capacity_fixture;
use super::fixture::Fixture;
use super::fixture_io::json;
use super::machine_tests::{register, report};
use ciproxy_protocol::agent::AgentMessage;

#[tokio::test]
async fn labels_use_current_capacity_keep_names_and_allow_delete_then_readd() {
    let mut fixture = Fixture::new();
    fixture.state.capacity = capacity_fixture::standard();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let (machine, _connection) = register(&fixture, &admin).await;
    let input = json!({"name":"custom","vcpus":2,"memory_gb":8,"disk_gb":20,"allow_machines":true,"allow_boat":true,"when_nothing_fits":"wait"});
    let preview = json(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/labels/preview"),
                Some(input.clone()),
            )
            .await,
    )
    .await;
    assert_eq!(preview["minimum_jobs"], "2");
    assert_eq!(preview["fitting_machines"], 1);
    let (_, newer) = register(&fixture, &admin).await;
    let mut observed = report();
    observed.hostname = "builder-new".to_owned();
    fixture
        .gateway
        .receive(newer.session, AgentMessage::Report { report: observed })
        .unwrap();
    fixture.state.capacity = capacity_fixture::mixed();
    let mixed = json(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/labels/preview"),
                Some(input.clone()),
            )
            .await,
    )
    .await;
    assert_eq!(mixed["eligible_machines"], 2);
    assert_eq!(mixed["fitting_machines"], 2);
    assert_eq!(mixed["minimum_jobs"], "1");
    assert_eq!(mixed["maximum_jobs"], "2");

    let response = fixture
        .request(
            Some(&admin),
            "POST",
            &fixture.path("/labels"),
            Some(input.clone()),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let saved = json(response).await;
    let id = saved["id"].as_str().unwrap();
    let path = fixture.path(&format!("/labels/{id}"));
    let mut renamed = input.clone();
    renamed["name"] = json!("renamed");
    assert_eq!(
        fixture
            .request(Some(&admin), "PUT", &path, Some(renamed))
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let mut bigger = input.clone();
    bigger["disk_gb"] = json!(100);
    assert_eq!(
        fixture
            .request(Some(&admin), "PUT", &path, Some(bigger))
            .await
            .status(),
        StatusCode::OK
    );
    fixture.remote.fail_delete.store(true, Ordering::SeqCst);
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &path, None)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(
        !fixture
            .database
            .store
            .labels(fixture.workspace)
            .unwrap()
            .iter()
            .any(|label| label.id.to_string() == id)
    );
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
            .iter()
            .any(|event| event.starts_with("delete:"))
    );
    let replacement = json(
        fixture
            .request(Some(&admin), "POST", &fixture.path("/labels"), Some(input))
            .await,
    )
    .await;
    assert_ne!(replacement["id"], saved["id"]);
    let labels = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/labels"), None)
            .await,
    )
    .await;
    assert_eq!(labels.as_array().unwrap().len(), 2);
    let default = labels
        .as_array()
        .unwrap()
        .iter()
        .find(|label| label["label"]["name"] == "ciproxy")
        .unwrap();
    let default_path = fixture.path(&format!("/labels/{}", default["id"].as_str().unwrap()));
    assert_eq!(
        fixture
            .request(Some(&admin), "DELETE", &default_path, None)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    let mut edited = default["label"].clone();
    edited["vcpus"] = json!(8);
    edited["memory_gb"] = json!(32);
    assert_eq!(
        fixture
            .request(Some(&admin), "PUT", &default_path, Some(edited))
            .await
            .status(),
        StatusCode::OK
    );
    let detail = json(
        fixture
            .request(
                Some(&admin),
                "GET",
                &fixture.path(&format!("/machines/{machine}")),
                None,
            )
            .await,
    )
    .await;
    assert_eq!(detail["concurrent_default_jobs"], "1");
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &fixture.path("/settings"),
                Some(json!({"name":"Renamed workspace"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let settings = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/settings"), None)
            .await,
    )
    .await;
    assert_eq!(settings["name"], "Renamed workspace");
    let setup = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/setup"), None)
            .await,
    )
    .await;
    assert_eq!(setup["waiting_for_machine"], false);
    assert_eq!(setup["current_image_disk_bytes"], "58300000000");
    assert!(setup["first_job"].is_null());
    fixture.state.capacity = capacity_fixture::unavailable();
    let detail = json(
        fixture
            .request(
                Some(&admin),
                "GET",
                &fixture.path(&format!("/machines/{machine}")),
                None,
            )
            .await,
    )
    .await;
    assert_eq!(detail["capacity_known"], false);
    assert!(detail["selected_image_disk_bytes"].is_null());
    assert!(detail["concurrent_default_jobs"].is_null());
}
