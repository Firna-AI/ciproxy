//! A fresh install learns its workspace from its token's authority.

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::secrets::TokenHash;
use uuid::Uuid;

use super::fixture::{Fixture, OTHER, WORKSPACE, at};
use super::messages::{register, report};
use super::socket::FakeAgent;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registers_without_workspace_in_the_install_token_owner_only() {
    let fixture = Fixture::start().await;
    let hash = TokenHash::new(vec![
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ]);
    fixture
        .database
        .store
        .issue_install_token(OTHER, Uuid::from_u128(50), &hash, at())
        .unwrap();
    let mut agent = FakeAgent::connect(&fixture.url).await;
    agent.send(register(Secret::new("abc"), report())).await;
    let ControlMessage::Registered {
        workspace_id,
        machine_id,
        ..
    } = agent.next().await
    else {
        panic!("expected registration from the install token alone");
    };
    assert_eq!(workspace_id.0, OTHER.to_string());
    let machines = fixture.database.store.machines(OTHER).unwrap();
    assert_eq!(machines.len(), 1);
    assert_eq!(machines[0].id.to_string(), machine_id.0);
    assert_eq!(fixture.database.store.machines(WORKSPACE).unwrap(), vec![]);
    agent.disconnect().await;
}
