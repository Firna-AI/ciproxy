//! A known image-limit refusal shows a fit of 0 jobs, with no disk size and
//! no advice to add disk.

use ciproxy_store_interface::dashboard::records::Role;
use serde_json::{Value, json};

use super::capacity_fixture;
use super::fixture::Fixture;
use super::fixture_io::json;
use super::machine_tests::register;

#[tokio::test]
async fn a_label_above_the_image_limit_fits_no_job_and_shows_no_disk_size() {
    let mut fixture = Fixture::new();
    fixture.state.capacity = capacity_fixture::above_image();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let _machine = register(&fixture, &admin).await;

    let saved = json(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/labels"),
                Some(json!({
                    "name": "large-disk", "platform": "linux_x64", "vcpus": 2,
                    "memory_gb": 8, "disk_gb": 40, "allow_machines": true,
                    "allow_boat": false, "when_nothing_fits": "wait"
                })),
            )
            .await,
    )
    .await;
    let fit = &saved["fit"];
    assert_eq!(fit["machines"][0]["concurrent_jobs"], "0", "{fit}");
    assert_eq!(fit["machines"][0]["required_root_disk_bytes"], Value::Null);
    assert_eq!(fit["unavailable_machines"], 0);

    let machines = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/machines"), None)
            .await,
    )
    .await;
    assert_eq!(machines[0]["default_root_disk_bytes"], Value::Null);
    assert_eq!(machines[0]["concurrent_default_jobs"], "0");
    assert_eq!(machines[0]["disk_limited"], false);
}
