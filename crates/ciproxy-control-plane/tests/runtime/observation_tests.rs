//! Lost start results retain execution evidence until later inventory releases capacity.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::registrations::RunnerRegistrationsMock;
use ciproxy_github::runtime::{Runtime, RuntimeMock};
use ciproxy_github::scale_sets::JitRegistration;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::reports::DrainState;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::recovery_support::{job, keep_warm, runtime_with_clock};
use super::support::{Fixture, at};

#[tokio::test]
async fn lost_start_reply_then_completed_github_job_releases_slots_and_disk_on_later_absence() {
    let mut fixture = Fixture::new();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Stopped,
            Some(47),
        )
        .unwrap();
    keep_warm(&fixture);
    fixture.contact.report.runners.clear();
    fixture.contact.report.capacity = Slots(1);
    fixture.contact.report.job_disk_bytes = 142_755_282_944;
    fixture.contact.report.drain = DrainState::Active;
    fixture.contact.at = at() - Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    let (clock, now) = clock();
    let next_registration = AtomicI64::new(48);
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::create
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                Ok(JitRegistration {
                    runner_id: next_registration.fetch_add(1, Ordering::SeqCst),
                    configuration: "jit-fixture".into(),
                })
            })),
    ));
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sent = messages.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                let offline = matches!(message, ControlMessage::JobPreparation { .. });
                sent.lock().unwrap().push(message);
                Ok(if offline {
                    Delivery::Offline
                } else {
                    Delivery::Connected
                })
            })),
    ));
    let runtime = runtime_with_clock(&fixture, registrations, gateway, clock);
    runtime.tick().await.unwrap();
    let issued = registered(&fixture, 48);
    assert_eq!(issued.runner.state, RunnerState::Starting);
    assert_eq!(issued.confirmed_at, None);
    assert!(!issued.acknowledged);
    assert_eq!(issued.slot_indices, vec![0]);
    assert_eq!(issued.runner.size.disk_bytes, 142_755_282_944);

    fixture.contact.at = at() + Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    let busy_at = at() + Duration::seconds(2);
    *now.lock().unwrap() = busy_at;
    let mut demand = job(&fixture);
    demand.runner_github_id = Some(48);
    demand.assigned_at = Some(at() - Duration::seconds(20));
    demand.started_at = Some(at() - Duration::seconds(10));
    fixture
        .store
        .save_github_job(fixture.workspace, &demand, busy_at)
        .unwrap();
    runtime
        .job(fixture.workspace, demand.clone())
        .await
        .unwrap();
    assert_eq!(registered(&fixture, 48).runner.state, RunnerState::Busy);
    runtime.tick().await.unwrap();
    assert_eq!(registered(&fixture, 48).runner.state, RunnerState::Busy);

    let completed_at = at() + Duration::seconds(3);
    *now.lock().unwrap() = completed_at;
    demand.phase = DemandPhase::Completed;
    demand.completed_at = Some(completed_at);
    demand.result = Some("success".into());
    fixture
        .store
        .save_github_job(fixture.workspace, &demand, completed_at)
        .unwrap();
    runtime.job(fixture.workspace, demand).await.unwrap();
    assert_eq!(registered(&fixture, 48).runner.state, RunnerState::Busy);

    fixture.contact.at = at() + Duration::seconds(4);
    *now.lock().unwrap() = fixture.contact.at;
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    runtime.tick().await.unwrap();
    let ended = registered(&fixture, 48);
    assert_eq!(ended.runner.state, RunnerState::Stopped);
    assert_eq!(ended.confirmed_at, Some(busy_at));
    assert_eq!(ended.command_id, issued.command_id);
    assert_eq!(ended.issued_at, issued.issued_at);
    assert!(!ended.acknowledged);
    let replacement = registered(&fixture, 49);
    assert_eq!(replacement.runner.state, RunnerState::Starting);
    assert_eq!(replacement.slot_indices, vec![0]);
    assert_eq!(replacement.runner.size.disk_bytes, 142_755_282_944);
    assert_eq!(replacement.confirmed_at, None);
    let starts: Vec<_> = messages
        .lock()
        .unwrap()
        .iter()
        .filter_map(|message| match message {
            ControlMessage::StartRunner { runner, .. } => Some(runner.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(starts.len(), 2);
    assert_eq!(starts[0].runner_id.0, issued.runner.id.to_string());
    assert_eq!(starts[1].runner_id.0, replacement.runner.id.to_string());
    assert_eq!(starts[1].slot_indices, vec![0]);
}

fn registered(fixture: &Fixture, github_id: i64) -> OwnRunner {
    fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.github_id == Some(github_id))
        .unwrap()
}

fn clock() -> (Arc<dyn Runtime>, Arc<Mutex<DateTime<Utc>>>) {
    let now = Arc::new(Mutex::new(at()));
    let observed = now.clone();
    let next_id = AtomicU64::new(100);
    let clock = Arc::new(Unimock::new((
        RuntimeMock::now
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| *observed.lock().unwrap())),
        RuntimeMock::uuid
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                Uuid::from_u128(u128::from(next_id.fetch_add(1, Ordering::SeqCst)))
            })),
    )));
    (clock, now)
}
