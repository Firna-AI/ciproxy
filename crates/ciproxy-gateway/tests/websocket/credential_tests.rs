//! A rejected credential cannot replace another machine's live connection.

use ciproxy_gateway::types::Delivery;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::{CommandId, MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::gateway::ReceivedReport;
use uuid::Uuid;

use super::fixture::{ADMIN, Fixture, WORKSPACE, at};
use super::messages::{hello, register, report};
use super::socket::FakeAgent;

fn registered(message: ControlMessage) -> (MachineId, Secret) {
    let ControlMessage::Registered {
        workspace_id,
        machine_id,
        credential,
    } = message
    else {
        panic!("expected successful registration");
    };
    assert_eq!(workspace_id.0, WORKSPACE.to_string());
    (machine_id, credential)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejects_another_machine_credential_without_changing_the_live_session_or_report() {
    let fixture = Fixture::start().await;
    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let original_report = report();
    let mut original = FakeAgent::connect(&fixture.url).await;
    original
        .send(register(token.token.clone(), original_report.clone()))
        .await;
    let (machine_id, original_credential) = registered(original.next().await);
    let machine_uuid = Uuid::parse_str(&machine_id.0).unwrap();
    let session = fixture
        .database
        .store
        .machine_session(WORKSPACE, machine_uuid)
        .unwrap()
        .unwrap();

    let mut other = FakeAgent::connect(&fixture.url).await;
    other.send(register(token.token, report())).await;
    let (other_id, other_credential) = registered(other.next().await);
    assert_ne!(other_id, machine_id);
    assert_ne!(other_credential, original_credential);
    other.disconnect().await;

    fixture.advance_to(30);
    let mut attempted_report = original_report.clone();
    attempted_report.hostname = "forged-hello-hostname".into();
    attempted_report.cache.used_bytes = 999_999;
    let mut rejected = FakeAgent::connect(&fixture.url).await;
    rejected
        .send(hello(
            WORKSPACE,
            machine_id,
            other_credential,
            1,
            attempted_report,
        ))
        .await;
    rejected.closed("[gateway] authentication failed").await;

    let snapshot = fixture
        .database
        .store
        .machine_snapshots(WORKSPACE)
        .unwrap()
        .into_iter()
        .find(|record| record.machine.id == machine_uuid)
        .unwrap();
    assert_eq!(
        snapshot.report,
        Some(ReceivedReport {
            report: original_report,
            received_at: at()
        })
    );
    assert_eq!(snapshot.machine.last_contact_at, Some(at()));
    assert_eq!(
        fixture
            .database
            .store
            .machine_session(WORKSPACE, machine_uuid)
            .unwrap(),
        Some(session)
    );

    let command = ControlMessage::StopRunner {
        command_id: CommandId("original-still-current".into()),
        organization_id: OrganizationId("org-1".into()),
        runner_id: RunnerId("warm".into()),
    };
    assert_eq!(
        fixture
            .gateway
            .send(WORKSPACE, machine_uuid, command.clone())
            .unwrap(),
        Delivery::Connected
    );
    assert_eq!(original.next().await, command);
    original.disconnect().await;
}
