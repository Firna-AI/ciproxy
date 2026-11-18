//! Admission and first-message deadlines at the public agent route.

use std::time::Duration;

use ciproxy_gateway::transport::TransportLimits;
use ciproxy_protocol::agent::ControlMessage;
use tokio_tungstenite::tungstenite::http::StatusCode;

use super::fixture::{ADMIN, Fixture, WORKSPACE};
use super::messages::{register, report};
use super::socket::FakeAgent;

pub(super) fn single_handshake_limits() -> TransportLimits {
    TransportLimits {
        first_message_timeout: Duration::from_secs(60),
        close_timeout: Duration::from_secs(60),
        max_pending_handshakes: 1,
        ..TransportLimits::default()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejects_excess_handshakes_until_authentication_releases_capacity() {
    let fixture = Fixture::start_with_limits(single_handshake_limits()).await;
    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut first = FakeAgent::connect(&fixture.url).await;
    assert_eq!(
        FakeAgent::upgrade_status(&fixture.url).await,
        StatusCode::SERVICE_UNAVAILABLE
    );

    first.send(register(token.token.clone(), report())).await;
    assert!(matches!(
        first.next().await,
        ControlMessage::Registered { .. }
    ));

    let mut second = FakeAgent::connect(&fixture.url).await;
    second.send(register(token.token, report())).await;
    assert!(matches!(
        second.next().await,
        ControlMessage::Registered { .. }
    ));
    first.disconnect().await;
    second.disconnect().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_message_timeout_closes_and_releases_handshake_capacity() {
    let limits = TransportLimits {
        first_message_timeout: Duration::from_millis(10),
        close_timeout: Duration::from_millis(10),
        ..single_handshake_limits()
    };
    let fixture = Fixture::start_with_limits(limits).await;
    let mut idle = FakeAgent::connect(&fixture.url).await;
    idle.closed("[gateway] handshake timed out").await;
    idle.server_ended_without_reply().await;

    assert_eq!(
        FakeAgent::upgrade_status(&fixture.url).await,
        StatusCode::SWITCHING_PROTOCOLS
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejects_an_oversized_first_frame_before_reading_its_payload() {
    let limits = TransportLimits {
        max_message_bytes: 4096,
        close_timeout: Duration::from_millis(10),
        ..single_handshake_limits()
    };
    let fixture = Fixture::start_with_limits(limits).await;
    let mut rejected = FakeAgent::connect(&fixture.url).await;
    rejected.announce_text_frame(4097).await;
    rejected.closed("[gateway] internal operation failed").await;
    rejected.server_ended_without_reply().await;
    assert_eq!(
        FakeAgent::upgrade_status(&fixture.url).await,
        StatusCode::SWITCHING_PROTOCOLS
    );
}
