//! Label route contract for default, reordered, and empty orders.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use serde_json::json;

use super::fixture::Fixture;
use super::fixture_io::json;

#[tokio::test]
async fn size_only_update_keeps_a_provider_offered_after_opening_the_label() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let created = fixture
        .request(
            Some(&admin),
            "POST",
            &fixture.path("/labels"),
            Some(json!({
                "name": "ordered", "vcpus": 2, "memory_gb": 8, "disk_gb": 20,
                "when_nothing_fits": "wait"
            })),
        )
        .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = json(created).await;
    assert_eq!(created["label"]["order"], json!([{"kind": "machine"}]));
    fixture
        .database
        .store
        .offer_overflow_provider(fixture.workspace, OverflowProvider::Boat)
        .unwrap();

    let mut body = created["label"].clone();
    body.as_object_mut().unwrap().remove("order");
    body["vcpus"] = json!(4);
    let path = fixture.path(&format!("/labels/{}", created["id"].as_str().unwrap()));
    let edited = fixture
        .request(Some(&admin), "PUT", &path, Some(body))
        .await;
    assert_eq!(edited.status(), StatusCode::OK);
    let edited = json(edited).await;
    assert_eq!(edited["label"]["vcpus"], 4);
    assert_eq!(
        edited["label"]["order"],
        json!([{"kind": "machine"}, {"kind": "overflow", "provider": "boat"}])
    );
}

#[tokio::test]
async fn label_routes_default_the_order_and_preserve_each_explicit_position() {
    let fixture = Fixture::new();
    fixture
        .database
        .store
        .offer_overflow_provider(fixture.workspace, OverflowProvider::Boat)
        .unwrap();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let mut body = json!({
        "name": "ordered", "vcpus": 2, "memory_gb": 8, "disk_gb": 20,
        "when_nothing_fits": "wait"
    });
    let created = fixture
        .request(
            Some(&admin),
            "POST",
            &fixture.path("/labels"),
            Some(body.clone()),
        )
        .await;
    assert_eq!(
        created.status(),
        StatusCode::OK,
        "create with default order"
    );
    let created = json(created).await;
    let machine = json!({"kind":"machine"});
    let overflow = json!({"kind":"overflow", "provider":"boat"});
    assert_eq!(created["label"]["order"], json!([machine, overflow]));

    let path = fixture.path(&format!("/labels/{}", created["id"].as_str().unwrap()));
    body["order"] = json!([overflow, machine]);
    body["when_nothing_fits"] = json!("largest_overflow");
    let edited = fixture
        .request(Some(&admin), "PUT", &path, Some(body.clone()))
        .await;
    assert_eq!(edited.status(), StatusCode::OK);
    let edited = json(edited).await;
    assert_eq!(edited["label"]["order"], body["order"]);
    assert_eq!(edited["label"]["when_nothing_fits"], "largest_overflow");

    let list = fixture
        .request(Some(&admin), "GET", &fixture.path("/labels"), None)
        .await;
    assert_eq!(list.status(), StatusCode::OK);
    let list = json(list).await;
    let saved = list
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == edited["id"])
        .unwrap();
    assert_eq!(saved["label"], edited["label"]);

    body["order"] = json!([]);
    let emptied = fixture
        .request(Some(&admin), "PUT", &path, Some(body))
        .await;
    assert_eq!(emptied.status(), StatusCode::OK);
    let emptied = json(emptied).await;
    assert_eq!(emptied["label"]["order"], json!([]));
    assert_eq!(
        emptied["fit"]["routing"],
        json!({"kind":"waits", "reason":{"kind":"empty_order"}})
    );
}
