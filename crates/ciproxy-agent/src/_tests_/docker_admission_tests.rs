//! Active Docker reservations hold bytes for both root admission and cache resize.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::cache::DockerDiskReport;
use ciproxy_protocol::identifiers::CommandId;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisksMock;
use crate::fixtures::{
    applied, controller, hardware, report_dependencies_for, spec, start_image, vm,
};
use crate::state::MachineState;

#[tokio::test]
async fn root_admission_waits_for_held_docker_bytes_then_reuses_the_released_capacity() {
    let held = Arc::new(AtomicU64::new(200_000_000_000));
    let measured = held.clone();
    let mut hardware = hardware();
    hardware.disks[0].capacity_bytes = 512_000_000_000;
    let mock = Unimock::new((
        report_dependencies_for(hardware),
        start_image(),
        DockerDisksMock::report
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                Ok(DockerDiskReport {
                    leased_bytes: measured.load(Ordering::SeqCst),
                    repositories: 1,
                    used_bytes: 1,
                })
            })),
        DockerDisksMock::reclaim
            .some_call(matching!(198_100_387_328))
            .returns(Ok(0)),
        CacheMock::budget
            .some_call(matching!(198_100_387_328))
            .returns(Ok(())),
        StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        VmsMock::allocate
            .some_call(matching!(_, _))
            .answers(&|_, report, _| {
                Ok(crate::state::VmRecord {
                    cache_permissions: None,
                    job_facts: None,
                    report,
                    ..vm("fresh", false)
                })
            }),
        VmsMock::start.some_call(matching!(_, _)).returns(Ok(())),
    ));
    let mut agent = controller(MachineState::default(), mock.clone());
    agent.dockers = Some(Arc::new(mock));
    let command = ControlMessage::StartRunner {
        command_id: CommandId("command-1".to_owned()),
        runner: spec(),
    };
    assert_eq!(
        agent.control(command.clone()).await.unwrap(),
        Some(AgentMessage::CommandResult {
            command_id: CommandId("command-1".to_owned()),
            outcome: CommandOutcome::Failed {
                reason: CommandFailure::RunnerStart
            }
        })
    );
    assert_eq!(agent.state.lock().await.runners.len(), 0);
    held.store(0, Ordering::SeqCst);
    assert_eq!(agent.control(command).await.unwrap(), applied());
    assert_eq!(agent.state.lock().await.runners.len(), 1);
}

#[tokio::test]
async fn periodic_reconciliation_keeps_attached_docker_space_out_of_the_cache_budget() {
    let budgets = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = budgets.clone();
    let mock = Unimock::new((
        crate::boundaries::HostMock::hardware
            .some_call(matching!())
            .returns(Ok(hardware())),
        DockerDisksMock::report
            .some_call(matching!())
            .returns(Ok(DockerDiskReport {
                leased_bytes: 100_000_000_000,
                repositories: 1,
                used_bytes: 10_000_000_000,
            })),
        DockerDisksMock::reclaim
            .some_call(matching!(_))
            .returns(Ok(20_000_000_000)),
        CacheMock::budget
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, bytes| {
                captured.lock().unwrap().push(bytes);
                Ok(())
            })),
    ));
    let mut agent = controller(MachineState::default(), mock.clone());
    agent.dockers = Some(Arc::new(mock));
    assert_eq!(
        agent
            .reconcile()
            .await
            .map_err(|_| "reserved Docker capacity was lost"),
        Ok(())
    );
    assert_eq!(*budgets.lock().unwrap(), vec![525_000_000_000]);
}
