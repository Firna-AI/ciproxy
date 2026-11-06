//! Old removal delivery cannot stop a runner reserved by the reinstallation.

use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::gateway::MachineGateway;
use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::recovery_support::{gateway, other_organization, runtime_with_clock};
use super::support::{Fixture, at};

#[tokio::test]
async fn reinstall_retires_only_old_reservations_github_first_and_keeps_busy_answers() {
    for removal in [
        RegistrationRetirement::Removed { id: 47 },
        RegistrationRetirement::Busy { id: 47 },
    ] {
        let fixture = Fixture::new();
        other_organization(&fixture);
        let mut organization = fixture
            .store
            .remove_github_installation(fixture.workspace, fixture.organization, 43, at())
            .unwrap();
        let event = fixture
            .store
            .pending_github_lifecycle(42)
            .unwrap()
            .remove(0)
            .event;
        organization.installation_id = 49;
        organization.app_state = AppState::Active;
        organization.removed_at = None;
        organization.delete_after = None;
        fixture.store.install_organization(&organization).unwrap();
        let mut new = fixture.intent.clone();
        new.runner.id = Uuid::new_v4();
        new.runner.github_id = Some(48);
        new.command_id = Uuid::new_v4();
        new.issued_at = at() + Duration::seconds(1);
        new.slot_indices = vec![1];
        fixture
            .store
            .create_own_runner(fixture.workspace, &new)
            .unwrap();
        let mut machine = fixture.store.machines(fixture.workspace).unwrap().remove(0);
        machine.draining = false;
        machine.warm_runners = 2;
        fixture
            .store
            .update_machine(fixture.workspace, &machine)
            .unwrap();
        let registrations = Arc::new(Unimock::new(
            RunnerRegistrationsMock::retire
                .some_call(matching!(_, _, Some(47)))
                .answers_arc(Arc::new(move |_, _, _, _| Ok(removal)))
                .at_least_times(0),
        ));
        let messages = Arc::new(Mutex::new(Vec::new()));
        let clock: Arc<dyn Runtime> = if removal == (RegistrationRetirement::Busy { id: 47 }) {
            Arc::new(Unimock::new(()))
        } else {
            clock(true)
        };
        let delivery: Arc<dyn MachineGateway> =
            if removal == (RegistrationRetirement::Busy { id: 47 }) {
                Arc::new(Unimock::new(()))
            } else {
                gateway(messages.clone())
            };
        let runtime = runtime_with_clock(&fixture, registrations, delivery, clock);
        runtime.lifecycle(event).await.unwrap();
        let runners = fixture.store.own_runners(fixture.workspace).unwrap();
        let old = runners
            .iter()
            .find(|runner| runner.runner.id == fixture.intent.runner.id)
            .unwrap();
        assert_eq!(
            old.runner.state,
            if removal == (RegistrationRetirement::Busy { id: 47 }) {
                RunnerState::Busy
            } else {
                RunnerState::Stopping
            }
        );
        let retained = runners
            .iter()
            .find(|runner| runner.runner.id == new.runner.id)
            .unwrap();
        assert_eq!(retained.runner.state, RunnerState::Ready);
        assert_eq!(retained.command_id, new.command_id);
        let stops: Vec<_> = messages
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| match message {
                ControlMessage::StopRunner { runner_id, .. } => Some(runner_id.0.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            stops,
            if removal == (RegistrationRetirement::Busy { id: 47 }) {
                vec![]
            } else {
                vec![fixture.intent.runner.id.to_string()]
            }
        );
    }
}
