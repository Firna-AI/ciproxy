//! Store-only revocation invalidates a connected agent on its next contact.

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};

use super::fixture::{ADMIN, Fixture, WORKSPACE, at};
use super::messages::{register, report};
use super::socket::FakeAgent;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tells_connected_agent_when_another_service_revokes_workspace_credentials() {
    let fixture = Fixture::start().await;
    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut agent = FakeAgent::connect(&fixture.url).await;
    agent.send(register(WORKSPACE, token.token, report())).await;
    let registered = agent.next().await;
    assert!(matches!(registered, ControlMessage::Registered { .. }));
    fixture
        .database
        .store
        .revoke_workspace_execution_access(WORKSPACE, at())
        .unwrap();
    agent.send(AgentMessage::Report { report: report() }).await;
    agent.closed("[gateway] authentication failed").await;
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap(), vec![]);
}
