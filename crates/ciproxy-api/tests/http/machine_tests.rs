//! Machine HTTP controls exercise the real gateway and persisted report fences.

use axum::http::StatusCode;
use chrono::Duration;
use ciproxy_gateway::types::Handshake;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::reports::MachineReport;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::json;
use uuid::Uuid;

use super::capacity_fixture;
use super::fixture::{Browser, Fixture};
use super::fixture_io::json;

pub(super) fn report() -> MachineReport {
    serde_json::from_value(json!({
        "hostname":"builder", "agent_version":"0.1.0", "hardware":{"processor":"CPU", "threads":16,"memory_bytes":64000000000u64,"kvm":true,"disks":[{"name":"/data","capacity_bytes":500000000000u64,"free_bytes":300000000000u64}]},
        "capacity":6,"job_disk_bytes":180_000_000_000u64,"slots":[{"index":0,"occupancy":{"state":"free"}}],"runners":[],"drain":"active",
        "cache":{"used_bytes":512,"capacity_bytes":100000,"restores_last_7_days":10,"hits_last_7_days":8,"jobs":[]},
        "docker_disks":{"repositories":2,"used_bytes":900,"leased_bytes":0},"images":{"available":[],"newest":null}
    })).unwrap()
}

pub(super) async fn register(fixture: &Fixture, browser: &Browser) -> (Uuid, Handshake) {
    let response = json(
        fixture
            .request(
                Some(browser),
                "POST",
                &fixture.path("/install-tokens"),
                None,
            )
            .await,
    )
    .await;
    let token = response["command"]
        .as_str()
        .unwrap()
        .split("--token ")
        .nth(1)
        .unwrap();
    let gateway = fixture.gateway.clone();
    let message = AgentMessage::Register {
        protocol_version: ProtocolVersion::CURRENT,
        install_token: Secret::new(token.to_owned()),
        report: report(),
    };
    let handshake = tokio::task::spawn_blocking(move || gateway.open(message))
        .await
        .unwrap()
        .unwrap();
    let machine = handshake.session.machine_id;
    (machine, handshake)
}

#[tokio::test]
async fn admin_controls_machine_and_clear_cache_targets_only_its_machine_entries() {
    let mut fixture = Fixture::new();
    fixture.state.capacity = capacity_fixture::standard();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let (id, mut handshake) = register(&fixture, &admin).await;
    let path = fixture.path(&format!("/machines/{id}"));
    let list = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/machines"), None)
            .await,
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let view = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(view["concurrent_default_jobs"], "2");
    assert_eq!(view["cache"]["hits_last_7_days"], "8");
    assert_eq!(view["cache"]["docker_disk_bytes"], "900");
    assert_eq!(view["occupancy"][0]["state"], "free");
    assert!(view["monthly_cost_cents"].is_null());
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &format!("{path}/warm-runners"),
                Some(json!({"warm_runners":3}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let warm = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(warm["warm_runners"], 3);
    assert!(warm["monthly_cost_cents"].is_null());
    assert!(
        fixture
            .database
            .store
            .dashboard_machine_costs(fixture.workspace)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &format!("{path}/cost"),
                Some(json!({"monthly_cost_cents":"0"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let free = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(free["monthly_cost_cents"], "0");
    assert_eq!(free["warm_runners"], 3);
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &format!("{path}/cost"),
                Some(json!({"monthly_cost_cents":"8899"}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let view = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(view["monthly_cost_cents"], "8899");
    assert_eq!(view["warm_runners"], 3);
    assert_eq!(view["cache"]["used_bytes"], "512");
    let history = fixture
        .database
        .store
        .dashboard_machine_costs(fixture.workspace)
        .unwrap()
        .len();
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &format!("{path}/warm-runners"),
                Some(json!({"warm_runners":4}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_machine_costs(fixture.workspace)
            .unwrap()
            .len(),
        history
    );
    let large = "100000000000000000000";
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &format!("{path}/cost"),
                Some(json!({"monthly_cost_cents":large}))
            )
            .await
            .status(),
        StatusCode::OK
    );
    let view = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(view["monthly_cost_cents"], large);
    assert_eq!(view["warm_runners"], 4);
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "PUT",
                &path,
                Some(json!({"warm_runners":3,"monthly_cost_cents":"8899"}))
            )
            .await
            .status(),
        StatusCode::METHOD_NOT_ALLOWED
    );

    for action in ["clear-cache", "drain", "resume"] {
        let response = fixture
            .request(
                Some(&admin),
                "POST",
                &format!("{path}/actions/{action}"),
                None,
            )
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let message = handshake.outbox.next().await.unwrap();
        assert!(matches!(
            (action, message),
            ("clear-cache", ControlMessage::ClearCache { .. })
                | ("drain", ControlMessage::Drain { .. })
                | ("resume", ControlMessage::Resume { .. })
        ));
    }
    fixture.advance(Duration::seconds(60));
    let view = json(fixture.request(Some(&admin), "GET", &path, None).await).await;
    assert_eq!(view["offline"], true);
    assert_eq!(view["health_old"], true);
    assert_eq!(view["cache"]["used_bytes"], "512");
    let response = fixture
        .request(
            Some(&admin),
            "POST",
            &format!("{path}/actions/remove"),
            None,
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(matches!(
        handshake.outbox.next().await,
        Some(ControlMessage::Remove { .. })
    ));
    assert_eq!(
        fixture
            .request(Some(&admin), "GET", &path, None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert!(
        fixture
            .database
            .store
            .machines(fixture.workspace)
            .unwrap()
            .is_empty()
    );
}
