//! Fragmented free slots remain usable and surviving VMs never move slots.

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::{CommandId, RunnerId};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::runners::RunnerExit;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::GuestProvenance;
use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::controller::Agent;
use crate::fixtures::{
    applied, controller, hardware, report_dependencies_for, spec, start_image, vm,
};
use crate::state::{MachineState, VmRecord};

#[tokio::test]
async fn fragmented_slots_are_shared_with_reports_and_reused_without_moving_survivors() {
    let mut first = vm("existing-a", false);
    first.report.slot_indices = vec![1];
    let mut second = vm("existing-b", false);
    second.report.slot_indices = vec![3];
    let mut hardware = hardware();
    hardware.threads = 12;
    hardware.memory_bytes = 48_000_000_000;
    let mock = Unimock::new((
        report_dependencies_for(hardware),
        start_image(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::allocate
            .each_call(matching!(_, _))
            .answers(&|_, report, _| {
                Ok(VmRecord {
                    cache_authority: CacheAuthority::Waiting,
                    job_facts: None,
                    local_id: format!("local-{}", report.runner_id.0),
                    report,
                    ..vm("allocated", false)
                })
            }),
        CacheMock::budget
            .each_call(matching!(402_301_161_984))
            .answers(&|_, _| Ok(())),
        VmsMock::start
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(())),
        VmsMock::stop.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
    ));
    let agent = controller(
        MachineState {
            runners: vec![first, second],
            ..MachineState::default()
        },
        mock,
    );
    let mut wide = spec();
    wide.size.vcpu = 4;
    wide.size.memory_bytes = 16_000_000_000;
    wide.slot_indices = vec![0, 2];
    assert_eq!(
        agent
            .control(ControlMessage::StartRunner {
                command_id: CommandId("command-1".to_owned()),
                runner: wide
            })
            .await
            .unwrap(),
        applied()
    );
    assert_eq!(
        agent.state.lock().await.runners[2].report.slot_indices,
        vec![0, 2]
    );
    agent
        .guest(
            GuestProvenance::InitialConfiguration,
            "existing-a",
            GuestToAgent::RunnerExited {
                runner_id: RunnerId("existing-a".to_owned()),
                exit: RunnerExit { code: Some(0) },
            },
        )
        .await
        .unwrap();
    let report = agent.report().await.unwrap();
    assert_eq!(report.slots[1].occupancy, SlotOccupancy::Free);
    assert_eq!(
        report.slots[3].occupancy,
        SlotOccupancy::Warm {
            runner_id: RunnerId("existing-b".to_owned())
        }
    );
    let mut next = spec();
    next.runner_id = RunnerId("next".to_owned());
    next.slot_indices = vec![1];
    assert_eq!(
        agent
            .control(ControlMessage::StartRunner {
                command_id: CommandId("command-1".to_owned()),
                runner: next
            })
            .await
            .unwrap(),
        applied()
    );
    let report = agent.report().await.unwrap();
    assert_eq!(
        report
            .slots
            .into_iter()
            .map(|slot| slot.occupancy)
            .collect::<Vec<_>>(),
        vec![
            SlotOccupancy::Warm {
                runner_id: RunnerId("runner-1".to_owned())
            },
            SlotOccupancy::Warm {
                runner_id: RunnerId("next".to_owned())
            },
            SlotOccupancy::Warm {
                runner_id: RunnerId("runner-1".to_owned())
            },
            SlotOccupancy::Warm {
                runner_id: RunnerId("existing-b".to_owned())
            },
        ]
    );
}
