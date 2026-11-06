//! Unavailable retirement stays pending; confirmed retirement saves stable stops.

use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::runners::RunnerState;

use super::recovery_support::{other_organization, saved};
use super::support::Fixture;
use super::takeout_gateway::Capture;
use super::takeout_installations::Probe;
use super::takeout_support::{Answer, Call, Remote, real_runtime};

#[tokio::test]
async fn unavailable_lookup_or_delete_keeps_takeout_pending_on_callback_and_tick() {
    for (id, answer, expected) in [
        (
            Some(47),
            Answer::DeleteUnavailable,
            vec![Call::Remove(47), Call::Remove(47)],
        ),
        (
            None,
            Answer::LookupUnavailable,
            vec![Call::Find, Call::Find],
        ),
    ] {
        let fixture = Fixture::new();
        other_organization(&fixture);
        fixture
            .store
            .update_runner(
                fixture.workspace,
                fixture.organization,
                fixture.intent.runner.id,
                RunnerState::Ready,
                id,
            )
            .unwrap();
        fixture
            .store
            .begin_organization_takeout(fixture.workspace, fixture.organization)
            .unwrap();
        let event = fixture
            .store
            .pending_github_lifecycle(42)
            .unwrap()
            .remove(0)
            .event;
        let remote = Remote::new(answer);
        let gateway = Capture::new(false);
        let mut runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
        let probe = Probe::present();
        runtime.installations = probe.clone();
        let result = runtime.lifecycle(event).await;
        assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
        runtime.tick().await.unwrap();
        assert_eq!(*remote.calls.lock().unwrap(), expected);
        assert_eq!(*probe.calls.lock().unwrap(), vec![43]);
        let held = saved(&fixture);
        assert_eq!(held.runner.state, RunnerState::Ready);
        assert_eq!(held.runner.github_id, id);
        assert_eq!(held.command_id, fixture.intent.command_id);
        assert_eq!(held.slot_indices, vec![0]);
        assert_eq!(held.runner.size, fixture.intent.runner.size);
        assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
        assert_eq!(
            gateway
                .messages
                .lock()
                .unwrap()
                .iter()
                .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
                .count(),
            0
        );
    }
}

#[tokio::test]
async fn confirmed_removal_or_absence_saves_a_stop_before_offline_delivery_and_replay() {
    for (id, answer, calls) in [
        (None, Answer::Removed, vec![Call::Find, Call::Remove(47)]),
        (None, Answer::Absent, vec![Call::Find]),
    ] {
        let fixture = Fixture::new();
        other_organization(&fixture);
        fixture
            .store
            .update_runner(
                fixture.workspace,
                fixture.organization,
                fixture.intent.runner.id,
                RunnerState::Ready,
                id,
            )
            .unwrap();
        fixture
            .store
            .begin_organization_takeout(fixture.workspace, fixture.organization)
            .unwrap();
        let event = fixture
            .store
            .pending_github_lifecycle(42)
            .unwrap()
            .remove(0)
            .event;
        let remote = Remote::new(answer);
        let gateway = Capture::new(true);
        let mut runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
        runtime.installations = Probe::present();
        runtime.lifecycle(event.clone()).await.unwrap();
        let stop = saved(&fixture);
        assert_eq!(stop.runner.state, RunnerState::Stopping);
        assert_eq!(
            stop.runner.github_id,
            if calls.len() == 2 { Some(47) } else { None }
        );
        runtime.lifecycle(event).await.unwrap();
        runtime.tick().await.unwrap();
        assert_eq!(*remote.calls.lock().unwrap(), calls);
        assert_eq!(saved(&fixture).command_id, stop.command_id);
        assert_eq!(saved(&fixture).issued_at, stop.issued_at);
        let sent: Vec<_> = gateway
            .messages
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| match message {
                ControlMessage::StopRunner { command_id, .. } => Some(command_id.0.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(sent, vec![stop.command_id.to_string(); 3]);
    }
}
