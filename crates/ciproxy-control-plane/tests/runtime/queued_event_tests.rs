//! Gateway callbacks retain ordered progress and reject replaced sessions.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use ciproxy_control_plane::hosted_events::Relay;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::boundary::AgentEvents;
use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::{AgentMessage, CommandOutcome};
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_store_interface::job_bindings::WorkerJobId;
use ciproxy_store_interface::runners::RunnerState;
use tokio::time::timeout;
use tracing::instrument::WithSubscriber;
use unimock::Unimock;
use uuid::Uuid;

use super::clock::clock;
use super::queued_event_support::{HeldBudget, lock_observer};
use super::recovery_support::{reconnect, runtime_with_clock, saved};
use super::support::{Fixture, at};

#[tokio::test]
async fn gateway_queue_saves_valid_runner_and_command_after_invalid_runner_event() {
    let fixture = Fixture::new();
    fixture
        .store
        .retire_own_runner(
            fixture.workspace,
            fixture.intent.runner.id,
            fixture.intent.command_id,
            at(),
            fixture.intent.runner.github_id,
        )
        .unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    assert!(!saved(&fixture).acknowledged);
    let runtime = Arc::new(runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        clock(false),
    ));
    let boundary: Arc<dyn OwnMachines> = runtime.clone();
    let relay = Relay::new();
    relay.bind(&boundary).unwrap();
    let session = Session {
        workspace_id: fixture.workspace,
        machine_id: fixture.machine,
        id: fixture.contact.session_id,
    };
    relay
        .receive(
            session,
            AgentMessage::RunnerReady {
                runner_id: RunnerId(Uuid::new_v4().to_string()),
            },
        )
        .unwrap();
    relay
        .receive(
            session,
            AgentMessage::JobStarted {
                runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                job_id: JobId("Worker/job:Opaque-Aa".into()),
            },
        )
        .unwrap();
    relay
        .receive(
            session,
            AgentMessage::CommandResult {
                command_id: CommandId(fixture.intent.command_id.to_string()),
                outcome: CommandOutcome::Applied,
            },
        )
        .unwrap();
    runtime.quiesce().await.unwrap();
    let current = saved(&fixture);
    assert_eq!(current.runner.state, RunnerState::Busy);
    assert_eq!(current.runner.github_id, Some(47));
    assert_eq!(current.command_id, fixture.intent.command_id);
    assert!(current.acknowledged);
    assert_eq!(current.confirmed_at, Some(at()));
    assert_eq!(current.slot_indices, vec![0]);
    assert_eq!(
        current.worker_job_id,
        Some(WorkerJobId(JobId("Worker/job:Opaque-Aa".into())))
    );
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn gateway_queue_ignores_runner_exit_from_replaced_session() {
    let (dispatcher, lock_polled, lock_closed) = lock_observer();
    async move {
        let fixture = Fixture::new();
        fixture
            .store
            .retire_own_runner(
                fixture.workspace,
                fixture.intent.runner.id,
                fixture.intent.command_id,
                at(),
                fixture.intent.runner.github_id,
            )
            .unwrap();
        fixture
            .store
            .update_runner(
                fixture.workspace,
                fixture.organization,
                fixture.intent.runner.id,
                RunnerState::Busy,
                Some(47),
            )
            .unwrap();
        assert!(!saved(&fixture).acknowledged);
        let held = HeldBudget::new();
        let runtime = Arc::new(runtime_with_clock(
            &fixture,
            Arc::new(Unimock::new(())),
            held.gateway.clone(),
            clock(true),
        ));
        let boundary: Arc<dyn OwnMachines> = runtime.clone();
        let relay = Relay::new();
        relay.bind(&boundary).unwrap();
        let ticking = runtime.clone();
        let tick = tokio::spawn(async move { ticking.tick().await });
        timeout(Duration::from_secs(10), held.entered)
            .await
            .unwrap()
            .unwrap();
        relay
            .receive(
                Session {
                    workspace_id: fixture.workspace,
                    machine_id: fixture.machine,
                    id: fixture.contact.session_id,
                },
                AgentMessage::RunnerExited {
                    runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                    exit: RunnerExit { code: Some(0) },
                },
            )
            .unwrap();
        timeout(Duration::from_secs(10), lock_polled)
            .await
            .unwrap()
            .unwrap();
        assert!(!lock_closed.load(Ordering::Acquire));
        reconnect(&fixture);
        let replacement = fixture
            .store
            .machine_session(fixture.workspace, fixture.machine)
            .unwrap()
            .unwrap();
        assert_ne!(replacement, fixture.contact.session_id);
        relay
            .receive(
                Session {
                    workspace_id: fixture.workspace,
                    machine_id: fixture.machine,
                    id: replacement,
                },
                AgentMessage::CommandResult {
                    command_id: CommandId(fixture.intent.command_id.to_string()),
                    outcome: CommandOutcome::Applied,
                },
            )
            .unwrap();
        held.release.send(()).unwrap();
        timeout(Duration::from_secs(10), tick)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        timeout(Duration::from_secs(10), runtime.quiesce())
            .await
            .unwrap()
            .unwrap();
        let current = saved(&fixture);
        assert_eq!(current.runner.state, RunnerState::Busy);
        assert_eq!(current.command_id, fixture.intent.command_id);
        assert!(current.acknowledged);
        assert_eq!(current.confirmed_at, Some(at()));
        assert_eq!(
            fixture
                .store
                .machine_session(fixture.workspace, fixture.machine)
                .unwrap(),
            Some(replacement)
        );
        runtime.shutdown().await.unwrap();
    }
    .with_subscriber(dispatcher)
    .await;
}
