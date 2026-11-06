//! Lost registration recovery preserves exact reservations and GitHub busy races.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::runners;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use super::clock::clock;
use super::support::{Fixture, at};

#[tokio::test]
async fn confirmed_configuration_loss_retires_github_first_with_exact_resources_and_preserves_busy()
{
    for busy in [false, true] {
        let mut fixture = Fixture::new();
        let mut label = fixture.store.labels(fixture.workspace).unwrap().remove(0);
        label.size.disk_gb = 40;
        fixture
            .store
            .update_label(fixture.workspace, &label)
            .unwrap();
        let mut machine = fixture.store.machines(fixture.workspace).unwrap().remove(0);
        machine.draining = false;
        machine.warm_runners = 1;
        fixture
            .store
            .update_machine(fixture.workspace, &machine)
            .unwrap();
        fixture
            .store
            .update_runner(
                fixture.workspace,
                fixture.organization,
                fixture.intent.runner.id,
                RunnerState::Starting,
                Some(47),
            )
            .unwrap();
        fixture.contact.report.drain = ciproxy_protocol::reports::DrainState::Active;
        fixture.contact.report.capacity.0 = 1;
        let report = &mut fixture.contact.report.runners[0];
        report.state = runners::RunnerState::Starting;
        report.configuration_lost = true;
        report.size.disk_bytes = 154_000_004_096;
        report.requested_size.disk_bytes = 40_000_000_000;
        report.slot_indices = vec![2];
        fixture
            .store
            .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let removed = events.clone();
        let stopped = events.clone();
        let clock = clock(true);
        let runtime = MachineRuntime::new(
            fixture.store.clone(),
            Arc::new(Unimock::new(
                MachineGatewayMock::send
                    .each_call(matching!(_, _, _))
                    .answers_arc(Arc::new(move |_, _, _, message| {
                        if matches!(message, ControlMessage::StopRunner { .. }) {
                            stopped.lock().unwrap().push("stop");
                        }
                        Ok(Delivery::Connected)
                    })),
            )),
            Arc::new(Unimock::new(
                RunnerRegistrationsMock::retire
                    .some_call(matching!(_, _, Some(47)))
                    .answers_arc(Arc::new(move |_, _, _, _| {
                        removed.lock().unwrap().push("github");
                        Ok(if busy {
                            RegistrationRetirement::Busy { id: 47 }
                        } else {
                            RegistrationRetirement::Removed { id: 47 }
                        })
                    })),
            )),
            Arc::new(AssignmentService {
                store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
            }),
            clock,
            None,
            Arc::new(Unimock::new(())),
        );
        runtime.tick().await.unwrap();
        let held = fixture
            .store
            .own_runners(fixture.workspace)
            .unwrap()
            .remove(0);
        assert_eq!(held.runner.size.disk_bytes, 154_000_004_096);
        assert_eq!(held.requested_size.disk_gb, 40);
        assert_eq!(held.slot_indices, vec![2]);
        assert_eq!(held.confirmed_at, Some(at()));
        assert_eq!(
            held.runner.state,
            if busy {
                RunnerState::Busy
            } else {
                RunnerState::Stopping
            }
        );
        assert_eq!(
            *events.lock().unwrap(),
            if busy {
                vec!["github"]
            } else {
                vec!["github", "stop"]
            }
        );
        assert_eq!(
            fixture.store.own_runners(fixture.workspace).unwrap().len(),
            1
        );
    }
}
