//! Payload overload stays lossy, and full reports recover exact Worker bindings.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use ciproxy_control_plane::hosted_events::Relay;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::boundary::AgentEvents;
use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::runners;
use ciproxy_store_interface::job_bindings::WorkerJobId;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use tokio::time::timeout;
use tracing::instrument::WithSubscriber;
use unimock::Unimock;
use uuid::Uuid;

use super::clock::clock;
use super::queued_event_support::{HeldBudget, PayloadObservation, payload_observer};
use super::recovery_support::{runtime_with_clock, saved};
use super::settled::Settled;
use super::support::Fixture;

#[tokio::test]
async fn payload_budget_holds_inflight_ids_and_full_report_recovers_the_dropped_worker() {
    let PayloadObservation {
        dispatcher,
        polled,
        closed,
        dropped,
    } = payload_observer();
    async move {
        let mut fixture = Fixture::new();
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
        let held = HeldBudget::with_sends(2);
        let runtime = Arc::new(runtime_with_clock(
            &fixture,
            Arc::new(Unimock::new(())),
            held.gateway.clone(),
            clock(true),
        ));
        let boundary: Arc<dyn OwnMachines> = runtime.clone();
        let relay = Relay::new();
        relay.bind(&boundary).unwrap();
        let session = Session {
            workspace_id: fixture.workspace,
            machine_id: fixture.machine,
            id: fixture.contact.session_id,
        };
        let ticking = runtime.clone();
        let tick = tokio::spawn(async move { ticking.tick().await });
        timeout(Duration::from_secs(10), held.entered)
            .await
            .unwrap()
            .unwrap();
        relay
            .receive(
                session,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(Uuid::new_v4().to_string()),
                    job_id: JobId("a".repeat(8 * 1024 * 1024)),
                },
            )
            .unwrap();
        timeout(Duration::from_secs(10), polled)
            .await
            .unwrap()
            .unwrap();
        assert!(!closed.load(Ordering::Acquire));
        relay
            .receive(
                session,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(Uuid::new_v4().to_string()),
                    job_id: JobId("b".repeat(8 * 1024 * 1024)),
                },
            )
            .unwrap();
        for _ in 0..2 {
            relay
                .receive(
                    session,
                    AgentMessage::JobStarted {
                        runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                        job_id: JobId("Worker/report:Opaque-Aa".into()),
                    },
                )
                .unwrap();
        }
        assert_eq!(dropped.load(Ordering::Acquire), 2);
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
        assert_eq!(saved(&fixture).worker_job_id, None);
        assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);

        fixture.contact.report.runners[0].state = runners::RunnerState::Running {
            job_id: JobId("Worker/report:Opaque-Aa".into()),
        };
        fixture
            .store
            .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
            .unwrap();
        runtime.settled_tick().await.unwrap();
        assert_eq!(
            saved(&fixture).worker_job_id,
            Some(WorkerJobId(JobId("Worker/report:Opaque-Aa".into())))
        );

        let mut peer = fixture.intent.clone();
        peer.runner.id = Uuid::new_v4();
        peer.runner.github_id = Some(48);
        peer.runner.place = RunnerPlace::Machine {
            machine_id: fixture.machine,
            first_slot: 1,
        };
        peer.slot_indices = vec![1];
        peer.command_id = Uuid::new_v4();
        fixture
            .store
            .create_own_runner(fixture.workspace, &peer)
            .unwrap();
        relay
            .receive(
                session,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(peer.runner.id.to_string()),
                    job_id: JobId("r".repeat(8 * 1024 * 1024 + 1)),
                },
            )
            .unwrap();
        timeout(Duration::from_secs(10), runtime.quiesce())
            .await
            .unwrap()
            .unwrap();
        let peer = fixture
            .store
            .own_runners(fixture.workspace)
            .unwrap()
            .into_iter()
            .find(|runner| runner.runner.id == peer.runner.id)
            .unwrap();
        assert_eq!(peer.runner.state, RunnerState::Busy);
        assert_eq!(
            peer.worker_job_id.as_ref().map(|id| id.0.0.len()),
            Some(8 * 1024 * 1024 + 1)
        );
        assert!(
            peer.worker_job_id
                .unwrap()
                .0
                .0
                .bytes()
                .all(|byte| byte == b'r')
        );
        assert_eq!(dropped.load(Ordering::Acquire), 2);
        runtime.shutdown().await.unwrap();
    }
    .with_subscriber(dispatcher)
    .await;
}
