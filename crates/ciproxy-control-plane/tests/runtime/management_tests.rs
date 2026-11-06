//! Machine removal revokes first and re-enters the unlocked shared gateway.

use std::sync::{Arc, OnceLock};

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::boundary::{AgentEventsMock, AuthorizationMock, ClockMock};
use ciproxy_gateway::connections::LocalConnections;
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::types::{Delivery, MachineAction};
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{Removal, RunnerRegistrationsMock};
use ciproxy_store_interface::runners::RunnerState;
use tokio::runtime::Handle;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::support::{Fixture, at};

#[tokio::test]
async fn offline_remove_waits_for_registration_retirement_after_revocation_without_holding_gateway_lock()
 {
    let fixture = Fixture::new();
    let workspace_id = fixture.workspace;
    let machine_id = fixture.machine;
    let runtime_slot = Arc::new(OnceLock::<Arc<dyn OwnMachines>>::new());
    let callback = runtime_slot.clone();
    let store = fixture.store.clone();
    let received = Handle::current();
    let events = Arc::new(Unimock::new(
        AgentEventsMock::machine_removed
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, workspace, machine| {
                assert_eq!((workspace, machine), (workspace_id, machine_id));
                assert_eq!(store.machine_session(workspace, machine).unwrap(), None);
                match received.block_on(callback.get().unwrap().machine_removed(workspace, machine))
                {
                    Ok(()) => Ok(()),
                    Err(error) => Err(ciproxy_gateway::error::Error::internal(error)),
                }
            })),
    ));
    let gateway: Arc<dyn MachineGateway> = Arc::new(Gateway::new(
        fixture.store.clone(),
        Arc::new(Unimock::new(
            AuthorizationMock::require_admin
                .some_call(matching!(_, _))
                .returns(Ok(())),
        )),
        Arc::new(Unimock::new(
            ClockMock::now.some_call(matching!()).returns(at()),
        )),
        Arc::new(Unimock::new(
            ciproxy_gateway::boundary::EntropyMock::identifier
                .some_call(matching!())
                .returns(Ok(Uuid::from_u128(777))),
        )),
        Arc::new(LocalConnections::default()),
        events,
    ));
    let clock = clock(true);
    let runtime: Arc<dyn OwnMachines> = Arc::new(MachineRuntime::new(
        fixture.store.clone(),
        gateway.clone(),
        Arc::new(Unimock::new(
            RunnerRegistrationsMock::remove
                .some_call(matching!(_, 47))
                .returns(Ok(Removal::Removed)),
        )),
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
    ));
    assert!(runtime_slot.set(runtime).is_ok());
    let result = tokio::task::spawn_blocking(move || {
        gateway.manage(
            Uuid::from_u128(778),
            workspace_id,
            machine_id,
            MachineAction::Remove,
        )
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(result, Delivery::Offline);
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Stopping
    );
    assert_eq!(fixture.store.machines(fixture.workspace).unwrap().len(), 0);
}
