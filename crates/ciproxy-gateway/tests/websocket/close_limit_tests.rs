//! Close ordering and bounded drain retain admission permits until completion.

use std::time::Duration;

use ciproxy_gateway::transport::TransportLimits;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::secret::Secret;
use tokio_tungstenite::tungstenite::http::StatusCode;

use super::fixture::{ADMIN, Fixture, WORKSPACE};
use super::messages::{register, report};
use super::socket::FakeAgent;
use super::transport_limit_tests::single_handshake_limits;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_handshake_holds_capacity_until_the_peer_close_reply() {
    let fixture = Fixture::start_with_limits(single_handshake_limits()).await;
    let mut rejected = FakeAgent::connect(&fixture.url).await;
    rejected
        .send(register(Secret::new("dummy-invalid-token"), report()))
        .await;
    rejected.closed("[gateway] authentication failed").await;

    assert_eq!(
        FakeAgent::upgrade_status(&fixture.url).await,
        StatusCode::SERVICE_UNAVAILABLE
    );
    rejected.acknowledge_close().await;

    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut accepted = FakeAgent::connect(&fixture.url).await;
    accepted.send(register(token.token, report())).await;
    assert!(matches!(
        accepted.next().await,
        ControlMessage::Registered { .. }
    ));
    accepted.disconnect().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_timeout_releases_capacity_without_the_peer_close_reply() {
    let limits = TransportLimits {
        close_timeout: Duration::from_millis(10),
        ..single_handshake_limits()
    };
    let fixture = Fixture::start_with_limits(limits).await;
    let mut rejected = FakeAgent::connect(&fixture.url).await;
    assert_eq!(
        FakeAgent::upgrade_status(&fixture.url).await,
        StatusCode::SERVICE_UNAVAILABLE
    );
    rejected
        .send(register(Secret::new("dummy-invalid-token"), report()))
        .await;
    rejected.closed("[gateway] authentication failed").await;
    rejected.server_ended_without_reply().await;

    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut accepted = FakeAgent::connect(&fixture.url).await;
    accepted.send(register(token.token, report())).await;
    assert!(matches!(
        accepted.next().await,
        ControlMessage::Registered { .. }
    ));
    accepted.disconnect().await;
}
