//! Admission exercises occupied slots, job disk, cache reclaim, and drain.

use std::sync::Arc;

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::resources::GIGABYTE;
use ciproxy_protocol::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{
    applied, controller, hardware, report_dependencies_for, spec, start_image, vm,
};
use crate::state::{MachineState, Mode, VmRecord};

fn held(count: usize) -> Vec<VmRecord> {
    (0..count)
        .map(|index| {
            let mut record = vm(&format!("held-{index}"), false);
            record.network_index = index as u16 + 1;
            record.report.state = match index % 4 {
                0 => RunnerState::Starting,
                1 => RunnerState::Warm,
                2 => vm("running", true).report.state,
                _ => RunnerState::Stopping,
            };
            record
        })
        .collect()
}

fn start() -> ControlMessage {
    ControlMessage::StartRunner {
        command_id: CommandId("command-1".to_owned()),
        runner: spec(),
    }
}

#[tokio::test]
async fn admission_refuses_slot_only_or_disk_only_exhaustion_and_drain() {
    for (disk_gb, count, mode) in [
        (1000, 6, Mode::Active),
        (256, 3, Mode::Active),
        (1000, 0, Mode::Draining),
    ] {
        let mut hardware = hardware();
        hardware.disks[0].capacity_bytes = disk_gb * GIGABYTE;
        let state = MachineState {
            runners: held(count),
            mode,
            ..MachineState::default()
        };
        let agent = controller(
            state.clone(),
            Unimock::new(report_dependencies_for(hardware)),
        );
        assert_eq!(
            agent.control(start()).await.unwrap(),
            Some(AgentMessage::CommandResult {
                command_id: CommandId("command-1".to_owned()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::RunnerStart
                },
            }),
            "disk={disk_gb} held={count} mode={mode:?}"
        );
        assert_eq!(*agent.state.lock().await, state);
    }
}

#[tokio::test]
async fn a_fitting_vm_reclaims_cache_before_preparation_at_each_resource_boundary() {
    for (disk_gb, count, expected_count, expected_budget) in
        [(256, 2, 3, 12_000_000_000), (1000, 5, 6, 390_000_000_000)]
    {
        let mut hardware = hardware();
        hardware.disks[0].capacity_bytes = disk_gb * GIGABYTE;
        hardware.disks[0].free_bytes = 0;
        let mock = Unimock::new((
            report_dependencies_for(hardware),
            start_image(),
            VmsMock::allocate
                .some_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, report, used| {
                    assert_eq!(used.len(), count);
                    Ok(VmRecord {
                        report,
                        ..vm("new-local", false)
                    })
                })),
            StoreMock::save.some_call(matching!(_)).returns(Ok(())),
            CacheMock::budget
                .next_call(matching!(_))
                .answers_arc(Arc::new(move |_, bytes| {
                    assert_eq!(bytes, expected_budget);
                    Ok(())
                })),
            VmsMock::start.next_call(matching!(_, _)).returns(Ok(())),
        ));
        let agent = controller(
            MachineState {
                runners: held(count),
                ..MachineState::default()
            },
            mock,
        );
        assert_eq!(agent.control(start()).await.unwrap(), applied());
        assert_eq!(agent.state.lock().await.runners.len(), expected_count);
    }
}

#[tokio::test]
async fn reconciliation_uses_the_same_held_disk_budget() {
    let mut hardware = hardware();
    hardware.disks[0].capacity_bytes = 256 * GIGABYTE;
    let mock = Unimock::new((
        HostMock::hardware
            .some_call(matching!())
            .returns(Ok(hardware)),
        VmsMock::restore
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        CacheMock::budget
            .some_call(matching!(12_000_000_000))
            .returns(Ok(())),
    ));
    let mut runners = held(3);
    for record in &mut runners {
        record.report.state = RunnerState::Warm;
    }
    let agent = controller(
        MachineState {
            runners,
            ..MachineState::default()
        },
        mock,
    );
    agent.reconcile().await.unwrap();
}
