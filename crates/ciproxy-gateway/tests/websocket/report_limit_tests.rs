//! Valid busy-machine reports fit the production cap, while larger reports fail.

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::jobs::JobIdBinding;
use ciproxy_protocol::reports::{MachineReport, SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_protocol::runners::RunnerState;

use super::fixture::{ADMIN, Fixture, WORKSPACE};
use super::messages::{register, report};
use super::socket::FakeAgent;

fn busy_report() -> MachineReport {
    let mut report = report();
    report.capacity = Slots(32);
    report.slots = (0..32)
        .map(|index| SlotReport {
            index,
            occupancy: SlotOccupancy::Job {
                runner_id: RunnerId(format!("{index:036}")),
                job_id: JobId(format!("{index:036}")),
            },
        })
        .collect();
    report.runners = (0..32)
        .map(|index| {
            let mut runner = report.runners[0].clone();
            runner.runner_id = RunnerId(format!("{index:036}"));
            runner.slot_indices = vec![index];
            runner.state = RunnerState::Running {
                job_id: JobId(format!("{index:036}")),
            };
            runner
        })
        .collect();
    let cache = report.cache.as_mut().measured().unwrap();
    let mut sample = cache.jobs[0].clone();
    sample.organization_id.0 = "1".repeat(20);
    sample.repository_id.0 = "2".repeat(20);
    sample.assignment.run_id = "3".repeat(20);
    sample.assignment.runner_id = "4".repeat(20);
    cache.jobs = (0..32 * 7 * 24 * 4)
        .map(|index| {
            let mut job = sample.clone();
            let job_id = JobId(format!("{index:036}"));
            job.job_id = job_id.clone();
            job.assignment.job_ids = JobIdBinding {
                worker_job_id: job_id.clone(),
                scale_set_job_id: job_id,
            };
            job
        })
        .collect();
    report
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accepts_a_busy_report_and_rejects_valid_fragmented_json_over_the_message_cap() {
    let mut fixture = Fixture::start().await;
    let token = fixture.gateway.issue_token(ADMIN, WORKSPACE).unwrap();
    let mut agent = FakeAgent::connect(&fixture.url).await;
    agent.send(register(token.token, report())).await;
    assert!(matches!(
        agent.next().await,
        ControlMessage::Registered { .. }
    ));

    let busy = busy_report();
    let message = AgentMessage::Report {
        report: busy.clone(),
    };
    let bytes = serde_json::to_vec(&message).unwrap().len();
    eprintln!("busy report: 32 slots, 32 runners, 21504 cache jobs, {bytes} JSON bytes");
    agent.send(message).await;
    agent.fence(&mut fixture).await;
    let snapshots = fixture.database.store.machine_snapshots(WORKSPACE).unwrap();
    assert_eq!(snapshots[0].report.as_ref().unwrap().report, busy);

    let mut over = busy;
    over.hostname
        .push_str(&"x".repeat(16 * 1024 * 1024 + 1 - bytes));
    agent
        .send_fragmented(AgentMessage::Report { report: over })
        .await;
    agent.closed("[gateway] internal operation failed").await;
    agent.acknowledge_close().await;
    let snapshots = fixture.database.store.machine_snapshots(WORKSPACE).unwrap();
    assert_eq!(
        snapshots[0].report.as_ref().unwrap().report.hostname,
        "build-one"
    );
}
