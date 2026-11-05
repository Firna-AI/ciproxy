//! Actual outbound fencing after another gateway owner replaces the session.

use std::sync::Arc;

use ciproxy_gateway::boundary::EntropyMock;
use ciproxy_gateway::connections::{Connections, LocalConnections};
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::types::{Delivery, Session};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{CommandId, LabelName, OrganizationId, RunnerId};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_protocol::runners::RunnerSpec;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::gateway::GatewayStore;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::fixture::{ADMIN, Fixture, WORKSPACE};
use super::messages::{hello, register, report};
use super::socket::FakeAgent;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn closes_stale_socket_before_a_queued_runner_start_can_reach_it() {
    let fixture = Fixture::start().await;
    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut agent = FakeAgent::connect(&fixture.url).await;
    agent.send(register(token.token, report())).await;
    let ControlMessage::Registered {
        machine_id,
        credential,
        ..
    } = agent.next().await
    else {
        panic!("registration failed");
    };
    let id = Uuid::parse_str(&machine_id.0).unwrap();
    let old = Session {
        workspace_id: WORKSPACE,
        machine_id: id,
        id: fixture
            .database
            .store
            .machine_session(WORKSPACE, id)
            .unwrap()
            .unwrap(),
    };
    let store: Arc<dyn GatewayStore> = PgStoreFactory.connect(&fixture.database.url).unwrap();
    let connections: Arc<dyn Connections> = Arc::new(LocalConnections::default());
    let replacement: Arc<dyn MachineGateway> = Arc::new(Gateway::new(
        store,
        Arc::new(Unimock::new(())),
        fixture.clock.clone(),
        Arc::new(Unimock::new(
            EntropyMock::identifier
                .some_call(matching!())
                .returns(Ok(Uuid::from_u128(10_000))),
        )),
        connections,
        Arc::new(Unimock::new(())),
    ));
    let _connected = replacement
        .open(hello(WORKSPACE, machine_id, credential, 1, report()))
        .unwrap();
    let command = ControlMessage::StartRunner {
        command_id: CommandId("stale-start".into()),
        runner: RunnerSpec {
            runner_id: RunnerId("stale-runner".into()),
            organization_id: OrganizationId("org-1".into()),
            label: LabelName("ciproxy".into()),
            size: VmSize {
                vcpu: 2,
                memory_bytes: 8 * GIGABYTE,
                disk_bytes: 80 * GIGABYTE,
            },
            registration: Secret::new("dummy-stale-registration"),
        },
    };
    assert_eq!(
        fixture.connections.send(old, command).unwrap(),
        Delivery::Connected
    );
    agent.send(AgentMessage::Report { report: report() }).await;
    agent.closed("[gateway] connection replaced").await;
}
