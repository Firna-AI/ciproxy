//! A label keeps its platform, a macOS label never uses Boat, and machines show their platform.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::{Value, json};

use super::capacity_fixture;
use super::fixture::Fixture;
use super::fixture_io::json;
use super::machine_tests::register;

fn mac_label() -> Value {
    json!({
        "name": "mac-build", "platform": "macos_arm64", "vcpus": 2, "memory_gb": 8,
        "disk_gb": 20, "allow_machines": true, "allow_boat": false,
        "when_nothing_fits": "wait"
    })
}

#[tokio::test]
async fn macos_labels_refuse_boat_keep_their_platform_and_skip_linux_machines() {
    let mut fixture = Fixture::new();
    fixture.state.capacity = capacity_fixture::standard();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let _machine = register(&fixture, &admin).await;
    let labels = fixture.path("/labels");
    for (field, value) in [
        ("allow_boat", json!(true)),
        ("when_nothing_fits", json!("largest_boat")),
    ] {
        let mut refused = mac_label();
        refused[field] = value;
        let response = fixture
            .request(Some(&admin), "POST", &labels, Some(refused))
            .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{field}");
    }

    let response = fixture
        .request(Some(&admin), "POST", &labels, Some(mac_label()))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let saved = json(response).await;
    assert_eq!(saved["label"]["platform"], "macos_arm64");
    assert_eq!(saved["fit"]["eligible_machines"], 0, "the Linux machine");
    assert_eq!(saved["fit"]["boat"]["state"], "disabled");

    let path = fixture.path(&format!("/labels/{}", saved["id"].as_str().unwrap()));
    let mut linux = mac_label();
    linux["platform"] = json!("linux_x64");
    assert_eq!(
        fixture
            .request(Some(&admin), "PUT", &path, Some(linux))
            .await
            .status(),
        StatusCode::CONFLICT
    );

    let machines = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/machines"), None)
            .await,
    )
    .await;
    assert_eq!(machines[0]["platform"], "linux_x64");
    assert_eq!(machines[0]["virtualization"], "available");
    assert_eq!(machines[0]["vm_limit"], Value::Null);
}
