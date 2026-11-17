//! A held GitHub request cannot hold gateway reads or another workspace's work.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use ciproxy_control_plane::hosted_events::Relay;
use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::boundary::{ClockMock, EntropyMock};
use ciproxy_gateway::connections::{Connections, LocalConnections, Outbox};
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::types::{Delivery, Session};
use ciproxy_github::assignments::JobAssignmentsMock;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::agent::{
    AgentMessage, CommandOutcome, ControlMessage, JobPreparationOutcome,
};
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::held_remote::HeldRemote;
use super::progress_socket::Socket;
use super::progress_support::{peer, reserve_warm};
use super::support::{Fixture, at};

async fn next(outbox: &mut dyn Outbox, preparation: bool) -> ControlMessage {
    loop {
        let message = outbox.next().await.unwrap();
        if matches!(message, ControlMessage::JobPreparation { .. }) == preparation
            && matches!(
                message,
                ControlMessage::StartRunner { .. } | ControlMessage::JobPreparation { .. }
            )
        {
            return message;
        }
    }
}

#[tokio::test]
async fn held_github_create_allows_gateway_report_command_and_other_workspace_start_and_preparation()
 {
    let mut fixture = Fixture::new();
    reserve_warm(&fixture);
    let reporter = fixture.add_machine();
    fixture
        .store
        .set_machine_draining(fixture.workspace, reporter, true)
        .unwrap();
    let relay = Arc::new(Relay::new());
    let connections = Arc::new(LocalConnections::default());
    let authorization = Unimock::new(());
    let gateway_clock = Unimock::new(
        ClockMock::now
            .each_call(matching!())
            .returns(at() + chrono::Duration::seconds(2)),
    );
    let entropy = Unimock::new(
        EntropyMock::identifier
            .each_call(matching!())
            .answers(&|_| Ok(Uuid::new_v4())),
    );
    let assignments = Unimock::new(
        JobAssignmentsMock::facts
            .each_call(matching!(_, _, _))
            .answers(&|_, _, _, _| {
                Ok(JobPreparationOutcome::GitHubOnly {
                    organization_id: OrganizationId("142".into()),
                })
            }),
    );
    let runtime_clock = Unimock::new((
        RuntimeMock::now.each_call(matching!()).returns(at()),
        RuntimeMock::uuid
            .each_call(matching!())
            .answers(&|_| Uuid::new_v4()),
    ));
    let installations = Unimock::new(());
    {
        let gateway: Arc<dyn MachineGateway> = Arc::new(Gateway::new(
            fixture.store.clone(),
            Arc::new(authorization.clone()),
            Arc::new(gateway_clock.clone()),
            Arc::new(entropy.clone()),
            connections.clone(),
            relay.clone(),
        ));
        let remote = HeldRemote::new(
            fixture.workspace,
            true,
            RegistrationRetirement::Removed { id: 100 },
        );
        let runtime = Arc::new(MachineRuntime::new(
            fixture.store.clone(),
            gateway.clone(),
            remote.clone(),
            Arc::new(assignments.clone()),
            Arc::new(runtime_clock.clone()),
            None,
            Arc::new(installations.clone()),
        ));
        let boundary: Arc<dyn OwnMachines> = runtime.clone();
        relay.bind(&boundary).unwrap();
        let own = Session {
            workspace_id: fixture.workspace,
            machine_id: fixture.machine,
            id: fixture.contact.session_id,
        };
        let mut own_outbox = connections.attach(own).unwrap();
        runtime.tick().await.unwrap();
        remote.entered.notified().await;
        let mut order = vec!["remote entered"];
        let mut initial = fixture.contact.report.clone();
        initial.runners.clear();
        let mut reports = Socket::open(
            &fixture,
            reporter,
            initial.clone(),
            gateway.clone(),
            connections.clone(),
        )
        .await;
        let mut report = initial;
        report.hostname = "saved while GitHub waits".into();
        reports
            .send(AgentMessage::RunnerReady {
                runner_id: RunnerId("unknown-runner".into()),
            })
            .await;
        reports
            .send(AgentMessage::CommandResult {
                command_id: CommandId(Uuid::new_v4().to_string()),
                outcome: CommandOutcome::Applied,
            })
            .await;
        reports
            .send(AgentMessage::Report {
                report: report.clone(),
            })
            .await;
        let report_saved = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            reports.report_saved.notified(),
        )
        .await;
        if report_saved.is_err() {
            remote.release.add_permits(1);
        }
        assert!(
            report_saved.is_ok(),
            "gateway read path must save the full report while GitHub remains held"
        );
        let command = ControlMessage::DockerBudget {
            command_id: CommandId("independent-budget".into()),
            budget: None,
        };
        assert_eq!(
            gateway
                .send(fixture.workspace, reporter, command.clone())
                .unwrap(),
            Delivery::Connected
        );
        assert_eq!(
            fixture
                .store
                .machine_snapshots(fixture.workspace)
                .unwrap()
                .into_iter()
                .find(|machine| machine.machine.id == reporter)
                .unwrap()
                .report
                .unwrap()
                .report,
            report
        );
        order.push("report saved");
        assert_eq!(reports.next().await, command);
        order.push("command delivered");
        let gateway_read = gateway.clone();
        let report = {
            let mut report = fixture.contact.report.clone();
            report.runners.clear();
            report
        };
        tokio::task::spawn_blocking(move || {
            gateway_read.receive(own, AgentMessage::Report { report })
        })
        .await
        .unwrap()
        .unwrap();
        let peer = peer(&fixture);
        let mut peer_outbox = connections.attach(peer.session).unwrap();
        runtime.job(peer.workspace, peer.job.clone()).await.unwrap();
        assert!(
            matches!(next(&mut *peer_outbox, true).await, ControlMessage::JobPreparation { runner_id, .. } if runner_id.0 == peer.runner.runner.id.to_string())
        );
        order.push("peer prepared");
        runtime.tick().await.unwrap();
        assert!(matches!(
            next(&mut *peer_outbox, false).await,
            ControlMessage::StartRunner { .. }
        ));
        order.push("peer started");
        assert!(!remote.released.load(Ordering::SeqCst));
        assert_eq!(remote.creates.load(Ordering::SeqCst), 2);
        order.push("remote released");
        remote.release.add_permits(1);
        runtime.quiesce().await.unwrap();
        assert!(matches!(
            next(&mut *own_outbox, false).await,
            ControlMessage::StartRunner { .. }
        ));
        assert_eq!(
            order,
            vec![
                "remote entered",
                "report saved",
                "command delivered",
                "peer prepared",
                "peer started",
                "remote released"
            ]
        );
        reports.close().await;
        runtime.shutdown().await.unwrap();
    }
}
