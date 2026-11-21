//! Install-token revocation keeps machines, and offline removal remains durable.

use axum::http::StatusCode;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_store_interface::dashboard::records::Role;

use super::fixture::Fixture;
use super::fixture_io::json;
use super::machine_tests::report;

#[tokio::test]
async fn token_revocation_preserves_registered_machines_and_offline_remove_is_durable() {
    let fixture = Fixture::new();
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let command = json(
        fixture
            .request(Some(&admin), "POST", &fixture.path("/install-tokens"), None)
            .await,
    )
    .await;
    assert_eq!(command["expires_at"], "2026-11-06T13:00:00Z");
    let raw = command["command"]
        .as_str()
        .unwrap()
        .split("--token ")
        .nth(1)
        .unwrap()
        .to_owned();
    let mut machines = Vec::new();
    for _ in 0..2 {
        let gateway = fixture.gateway.clone();
        let token = raw.clone();
        let handshake = tokio::task::spawn_blocking(move || {
            gateway.open(AgentMessage::Register {
                protocol_version: ProtocolVersion::CURRENT,
                install_token: Secret::new(token),
                report: report(),
            })
        })
        .await
        .unwrap()
        .unwrap();
        machines.push(handshake.session.machine_id);
    }
    let id = command["id"].as_str().unwrap();
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "DELETE",
                &fixture.path(&format!("/install-tokens/{id}")),
                None
            )
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .database
            .store
            .machines(fixture.workspace)
            .unwrap()
            .len(),
        2
    );
    let gateway = fixture.gateway.clone();
    let result = tokio::task::spawn_blocking(move || {
        gateway.open(AgentMessage::Register {
            protocol_version: ProtocolVersion::CURRENT,
            install_token: Secret::new(raw),
            report: report(),
        })
    })
    .await
    .unwrap();
    assert!(matches!(
        result,
        Err(ciproxy_gateway::error::Error::Unauthorized)
    ));
    let response = json(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path(&format!("/machines/{}/actions/remove", machines[0])),
                None,
            )
            .await,
    )
    .await;
    assert_eq!(response["connected"], false);
    assert_eq!(
        fixture
            .database
            .store
            .machines(fixture.workspace)
            .unwrap()
            .len(),
        1
    );
}
