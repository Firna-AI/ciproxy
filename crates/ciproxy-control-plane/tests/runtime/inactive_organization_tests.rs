//! Local holds stop idle starts while preserving received work and active demand.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::registrations::RegisteredRunner;
use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::agent::{ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::identifiers::{OrganizationId, RunnerId};
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::inactive_support::populate;
use super::recovery_support::{gateway, runtime_with_clock};
use super::support::{Fixture, at};

#[tokio::test]
async fn inactive_organization_retires_only_idle_work_and_routes_received_jobs_with_active_siblings()
 {
    let mut fixture = Fixture::new();
    let (sibling, running, ready) = populate(&mut fixture);
    let organization = fixture
        .store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    let event = LifecycleEvent::Inactive {
        organization,
        billing_hold: false,
        admin_approval_hold: true,
    };
    let messages = Arc::new(Mutex::new(Vec::new()));
    let registered = Arc::new(AtomicI64::new(100));
    let next = registered.clone();
    let registrations = Arc::new(Unimock::new((
        RunnerRegistrationsMock::retire
            .each_call(matching!(_, _, _))
            .answers(&|_, _, _, saved| Ok(RegistrationRetirement::Removed { id: saved.unwrap() })),
        RunnerRegistrationsMock::create
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, request, _| {
                Ok(RegisteredRunner {
                    target: request.target,
                    runner_id: next.fetch_add(1, Ordering::SeqCst),
                    configuration: "fixture".into(),
                })
            })),
    )));
    let runtime = runtime_with_clock(
        &fixture,
        registrations,
        gateway(messages.clone()),
        Arc::new(Unimock::new((
            RuntimeMock::now.each_call(matching!()).returns(at()),
            RuntimeMock::uuid
                .each_call(matching!())
                .answers(&|_| Uuid::new_v4()),
        ))),
    );
    let mut superseded = event.organization().clone();
    superseded.installation_id = 44;
    runtime
        .lifecycle(LifecycleEvent::Inactive {
            organization: superseded,
            billing_hold: false,
            admin_approval_hold: true,
        })
        .await
        .unwrap();
    assert_eq!(
        fixture
            .store
            .own_runners(fixture.workspace)
            .unwrap()
            .into_iter()
            .find(|runner| runner.runner.id == fixture.intent.runner.id)
            .unwrap()
            .runner
            .state,
        RunnerState::Ready,
        "an inactive event must name the current installation"
    );
    assert_eq!(messages.lock().unwrap().len(), 0);
    runtime.lifecycle(event.clone()).await.unwrap();
    let saved = fixture.store.own_runners(fixture.workspace).unwrap();
    assert_eq!(
        saved
            .iter()
            .find(|runner| runner.runner.id == fixture.intent.runner.id)
            .unwrap()
            .runner
            .state,
        RunnerState::Stopping,
        "inactive event must retire its idle runner"
    );
    assert_eq!(
        saved
            .iter()
            .find(|runner| runner.runner.id == running.runner.id)
            .unwrap()
            .runner
            .state,
        RunnerState::Busy
    );
    assert_eq!(
        saved
            .iter()
            .find(|runner| runner.runner.id == ready.runner.id)
            .unwrap(),
        &ready
    );
    runtime.tick().await.unwrap();
    runtime.lifecycle(event.clone()).await.unwrap();
    fixture.admit_organization();
    let mut admitted = fixture.intent.clone();
    admitted.runner.id = Uuid::new_v4();
    admitted.runner.github_id = Some(150);
    admitted.command_id = Uuid::new_v4();
    admitted.acknowledged = false;
    fixture
        .store
        .create_own_runner(fixture.workspace, &admitted)
        .unwrap();
    runtime.lifecycle(event).await.unwrap();
    assert_eq!(
        fixture
            .store
            .own_runners(fixture.workspace)
            .unwrap()
            .into_iter()
            .find(|runner| runner.runner.id == admitted.runner.id)
            .unwrap(),
        admitted,
        "stale inactive events cannot retire newly admitted runners"
    );
    let messages = messages.lock().unwrap();
    let starts: Vec<_> = messages
        .iter()
        .filter_map(|message| match message {
            ControlMessage::StartRunner { runner, .. } => Some(runner.organization_id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        starts,
        vec![OrganizationId(sibling.github_id.to_string())],
        "only the admitted sibling receives new warm capacity"
    );
    assert_eq!(registered.load(Ordering::SeqCst), 101);
    let prepared: Vec<_> = messages
        .iter()
        .filter_map(|message| match message {
            ControlMessage::JobPreparation {
                runner_id,
                outcome: JobPreparationOutcome::TrustedFacts { facts },
                ..
            } => Some((
                runner_id.clone(),
                facts.organization_id.clone(),
                facts.assignment.runner_id.clone(),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        prepared,
        vec![
            (
                RunnerId(running.runner.id.to_string()),
                OrganizationId("42".into()),
                "49".to_owned()
            );
            3
        ]
    );
    let stops: Vec<_> = messages
        .iter()
        .filter_map(|message| match message {
            ControlMessage::StopRunner { runner_id, .. } => Some(runner_id.clone()),
            _ => None,
        })
        .collect();
    assert!(!stops.is_empty());
    assert!(
        stops
            .iter()
            .all(|runner| runner.0 == fixture.intent.runner.id.to_string())
    );
    let saved = fixture.store.own_runners(fixture.workspace).unwrap();
    assert_eq!(
        saved
            .iter()
            .find(|runner| runner.runner.id == running.runner.id)
            .unwrap()
            .runner
            .state,
        RunnerState::Busy
    );
    assert_eq!(
        saved
            .iter()
            .find(|runner| runner.runner.id == running.runner.id)
            .unwrap()
            .runner
            .size
            .disk_bytes,
        142_755_282_944
    );
}
