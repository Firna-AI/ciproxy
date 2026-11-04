//! Registration, reconnect, reports, commands, and revocation over real WebSockets.

use ciproxy_gateway::types::{Delivery, MachineAction};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{CommandId, MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::reports::DrainState;
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use uuid::Uuid;

use super::fixture::{ADMIN, Fixture, OTHER, WORKSPACE, at};
use super::messages::{hello, register, report, version};
use super::socket::FakeAgent;

fn identity(message: ControlMessage) -> (MachineId, Secret) {
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
async fn serves_machine_lifecycle_without_losing_identity_or_latest_health() {
    let mut fixture = Fixture::start().await;
    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut first = FakeAgent::connect(&fixture.url).await;
    let mut no_kvm = report();
    no_kvm.hardware.kvm = false;
    no_kvm.capacity = Slots(0);
    no_kvm.slots.clear();
    no_kvm.runners.clear();
    first.send(register(token.token.clone(), no_kvm)).await;
    let (first_id, first_credential) = identity(first.next().await);
    let first_uuid = Uuid::parse_str(&first_id.0).unwrap();
    let machine = fixture
        .database
        .store
        .machines(WORKSPACE)
        .unwrap()
        .remove(0);
    assert_eq!((machine.kvm_available, machine.slots), (false, 0));
    first.disconnect().await;

    fixture.advance_to(3599);
    let mut second = FakeAgent::connect(&fixture.url).await;
    second.send(register(token.token.clone(), report())).await;
    let (second_id, credential) = identity(second.next().await);
    let second_uuid = Uuid::parse_str(&second_id.0).unwrap();
    assert_ne!(first_id, second_id);
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap().len(), 2);

    fixture.advance_to(3600);
    let mut expired = FakeAgent::connect(&fixture.url).await;
    expired.send(register(token.token, report())).await;
    expired.closed("[gateway] authentication failed").await;
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap().len(), 2);

    let live_token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut third = FakeAgent::connect(&fixture.url).await;
    third
        .send(register(live_token.token.clone(), report()))
        .await;
    let (third_id, third_credential) = identity(third.next().await);
    fixture
        .gateway
        .revoke_token(ADMIN, WORKSPACE, live_token.id)
        .unwrap();
    let mut revoked = FakeAgent::connect(&fixture.url).await;
    revoked.send(register(live_token.token, report())).await;
    revoked.closed("[gateway] authentication failed").await;
    third.disconnect().await;
    let mut retained = FakeAgent::connect(&fixture.url).await;
    retained
        .send(hello(WORKSPACE, third_id, third_credential, 1, report()))
        .await;
    assert_eq!(
        retained.next().await,
        ControlMessage::Welcome {
            protocol_version: ProtocolVersion::CURRENT
        }
    );

    let mut wrong_workspace = FakeAgent::connect(&fixture.url).await;
    wrong_workspace
        .send(hello(
            OTHER,
            second_id.clone(),
            credential.clone(),
            1,
            report(),
        ))
        .await;
    wrong_workspace
        .closed("[gateway] authentication failed")
        .await;
    let mut wrong_version = FakeAgent::connect(&fixture.url).await;
    wrong_version
        .send(hello(
            WORKSPACE,
            second_id.clone(),
            credential.clone(),
            2,
            report(),
        ))
        .await;
    wrong_version
        .closed("[gateway] unsupported protocol version")
        .await;

    fixture
        .gateway
        .manage(ADMIN, WORKSPACE, second_uuid, MachineAction::Drain)
        .unwrap();
    assert!(matches!(second.next().await, ControlMessage::Drain { .. }));
    let mut previous = FakeAgent::connect(&fixture.url).await;
    previous
        .send(hello(
            WORKSPACE,
            second_id.clone(),
            credential.clone(),
            0,
            report(),
        ))
        .await;
    assert_eq!(
        previous.next().await,
        ControlMessage::Welcome {
            protocol_version: version(0)
        }
    );
    assert!(matches!(
        previous.next().await,
        ControlMessage::Drain { .. }
    ));
    second.closed("[gateway] connection replaced").await;

    fixture.advance_to(3610);
    let mut latest = report();
    latest.hostname = "renamed-builder".into();
    latest.cache.used_bytes = 42_000;
    previous
        .send(AgentMessage::Report {
            report: latest.clone(),
        })
        .await;
    previous.fence(&mut fixture).await;
    let stored = fixture
        .database
        .store
        .machine_snapshots(WORKSPACE)
        .unwrap()
        .into_iter()
        .find(|record| record.machine.id == second_uuid)
        .unwrap();
    assert_eq!(stored.report, Some(latest.clone()));
    assert_eq!(
        stored.machine.last_contact_at,
        Some(at() + chrono::Duration::seconds(3610))
    );
    assert!(stored.machine.draining);
    assert_eq!(stored.machine.warm_runners, 2);

    fixture.advance_to(3611);
    previous.fence(&mut fixture).await;
    let contact = fixture
        .database
        .store
        .machines(WORKSPACE)
        .unwrap()
        .into_iter()
        .find(|machine| machine.id == second_uuid)
        .unwrap()
        .last_contact_at;
    assert_eq!(contact, Some(at() + chrono::Duration::seconds(3611)));

    fixture
        .gateway
        .manage(ADMIN, WORKSPACE, second_uuid, MachineAction::Resume)
        .unwrap();
    assert!(matches!(
        previous.next().await,
        ControlMessage::Resume { .. }
    ));
    fixture
        .gateway
        .manage(ADMIN, WORKSPACE, second_uuid, MachineAction::ClearCache)
        .unwrap();
    assert!(matches!(
        previous.next().await,
        ControlMessage::ClearCache { .. }
    ));
    let command = ControlMessage::StopRunner {
        command_id: CommandId("stop-warm".into()),
        organization_id: OrganizationId("org-2".into()),
        runner_id: RunnerId("warm".into()),
    };
    assert_eq!(
        fixture
            .gateway
            .send(WORKSPACE, second_uuid, command.clone())
            .unwrap(),
        Delivery::Connected
    );
    assert_eq!(previous.next().await, command);
    previous.disconnect().await;

    let mut reinstall = FakeAgent::connect(&fixture.url).await;
    let mut stale_drain = latest.clone();
    stale_drain.drain = DrainState::Draining;
    reinstall
        .send(hello(
            WORKSPACE,
            second_id.clone(),
            credential.clone(),
            1,
            stale_drain,
        ))
        .await;
    assert_eq!(
        reinstall.next().await,
        ControlMessage::Welcome {
            protocol_version: ProtocolVersion::CURRENT
        }
    );
    assert!(matches!(
        reinstall.next().await,
        ControlMessage::Resume { .. }
    ));
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap().len(), 3);
    assert_eq!(
        fixture
            .gateway
            .manage(ADMIN, WORKSPACE, first_uuid, MachineAction::Remove)
            .unwrap(),
        Delivery::Offline
    );
    let mut offline = FakeAgent::connect(&fixture.url).await;
    offline
        .send(hello(WORKSPACE, first_id, first_credential, 1, report()))
        .await;
    offline.closed("[gateway] authentication failed").await;
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap().len(), 2);

    assert_eq!(
        fixture
            .gateway
            .manage(ADMIN, WORKSPACE, second_uuid, MachineAction::Remove)
            .unwrap(),
        Delivery::Connected
    );
    assert!(matches!(
        reinstall.next().await,
        ControlMessage::Remove { .. }
    ));
    let mut removed = FakeAgent::connect(&fixture.url).await;
    removed
        .send(hello(WORKSPACE, second_id, credential, 1, latest))
        .await;
    removed.closed("[gateway] authentication failed").await;
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap().len(), 1);
}
