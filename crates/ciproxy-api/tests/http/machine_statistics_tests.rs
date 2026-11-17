//! Independent statistics and legacy report projection through the HTTP router.

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::cache::{DockerDiskReport, MachineCacheReport};
use ciproxy_protocol::statistics::Statistics;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::{Value, json};
use uuid::Uuid;

use super::capacity_fixture;
use super::fixture::Fixture;
use super::fixture_io::json;
use super::machine_tests::{register, report};

#[tokio::test]
async fn machine_statistics_keep_independent_availability_and_legacy_report_semantics() {
    let mut fixture = Fixture::new();
    fixture.state.capacity = capacity_fixture::standard();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let (machine, connection) = register(&fixture, &admin).await;
    let path = fixture.path(&format!("/machines/{machine}"));
    let measured_cache = json!({
        "status":"measured", "used_bytes":"512", "capacity_bytes":"100000",
        "restores_last_7_days":"10", "hits_last_7_days":"8"
    });
    let measured_docker = json!({
        "status":"measured", "repositories":"2", "used_bytes":"900"
    });
    let list = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/machines"), None)
            .await,
    )
    .await;
    assert_eq!(
        list[0]["cache"],
        json!({
            "machine_cache":measured_cache, "docker_disks":measured_docker
        })
    );

    let legacy = report();
    for (cache, docker, expected) in [
        (
            Statistics::Unavailable,
            legacy.docker_disks.clone(),
            json!({"machine_cache":{"status":"unavailable"}, "docker_disks":measured_docker}),
        ),
        (
            legacy.cache.clone(),
            Statistics::Unavailable,
            json!({"machine_cache":measured_cache, "docker_disks":{"status":"unavailable"}}),
        ),
        (
            Statistics::Unavailable,
            Statistics::Unavailable,
            json!({"machine_cache":{"status":"unavailable"}, "docker_disks":{"status":"unavailable"}}),
        ),
        (
            Statistics::Measured(MachineCacheReport {
                used_bytes: 0,
                capacity_bytes: 0,
                restores_last_7_days: 0,
                hits_last_7_days: 0,
                unknown_jobs: vec![],
                jobs: vec![],
            }),
            Statistics::Measured(DockerDiskReport {
                repositories: 0,
                used_bytes: 0,
                leased_bytes: 0,
            }),
            json!({
                "machine_cache":{"status":"measured", "used_bytes":"0", "capacity_bytes":"0", "restores_last_7_days":"0", "hits_last_7_days":"0"},
                "docker_disks":{"status":"measured", "repositories":"0", "used_bytes":"0"}
            }),
        ),
    ] {
        let mut observed = report();
        observed.cache = cache;
        observed.docker_disks = docker;
        fixture
            .gateway
            .receive(
                connection.session,
                AgentMessage::Report { report: observed },
            )
            .unwrap();
        let view = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
        assert_eq!(view["cache"], expected);
    }

    let mut no_report = fixture
        .database
        .store
        .machines(fixture.workspace)
        .unwrap()
        .remove(0);
    no_report.id = Uuid::new_v4();
    no_report.hostname = "legacy-without-report".to_owned();
    no_report.last_contact_at = None;
    fixture
        .database
        .store
        .create_machine(fixture.workspace, &no_report)
        .unwrap();
    let view = json(
        fixture
            .request(
                Some(&admin),
                "GET",
                &fixture.path(&format!("/machines/{}", no_report.id)),
                None,
            )
            .await,
    )
    .await;
    assert_eq!(view["cache"], Value::Null);
}
