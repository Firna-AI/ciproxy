//! Retirement releases reports before network cleanup and retains restart ownership.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::runners::RunnerExit;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, Vms};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, report_dependencies, vm};
use crate::network_fixtures::{Deletion, HostNetworks, commands, restored, runtime, saving};
use crate::process::{CommandsMock, UnitState};
use crate::state::{MachineState, Mode};
use crate::terminal_completion::CompletionAction;
use crate::vm_files::VmFilesMock;
use crate::vsock::GuestsMock;

#[tokio::test]
async fn runner_exit_releases_slots_before_cleanup_and_recovers_network_ownership_after_restart() {
    let host = Arc::new(Mutex::new(HostNetworks {
        namespaces: ["cpx1".to_owned()].into(),
        deletion: Deletion::Fail,
        ..HostNetworks::default()
    }));
    let saved = Arc::new(Mutex::new(Vec::new()));
    let mock = Arc::new(Unimock::new((
        commands(host.clone()),
        saving(saved.clone()),
        report_dependencies(),
        GuestsMock::detach.some_call(matching!(_)).returns(Ok(())),
        CommandsMock::unit
            .some_call(matching!(_))
            .returns(Ok(UnitState::Absent)),
        VmFilesMock::remove.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        CacheMock::refresh_budget
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
    )));
    let vms = Arc::new(runtime(mock.clone()));
    let mut agent = controller(
        MachineState {
            runners: vec![vm("runner", true)],
            mode: Mode::Removed,
            ..MachineState::default()
        },
        mock.as_ref().clone(),
    );
    agent.vms = vms.clone();
    let response = agent
        .guest(
            GuestProvenance::Recovery,
            "runner",
            GuestToAgent::RunnerExited {
                runner_id: RunnerId("runner".to_owned()),
                exit: RunnerExit { code: Some(0) },
            },
        )
        .await
        .unwrap();
    assert_eq!(
        response,
        Some(AgentMessage::RunnerExited {
            runner_id: RunnerId("runner".to_owned()),
            exit: RunnerExit { code: Some(0) }
        })
    );
    assert_eq!(agent.state.lock().await.pending_networks, [1].into());
    assert_eq!(host.lock().unwrap().deletions.len(), 0);
    let report = agent.report().await.unwrap();
    assert_eq!(report.runners.len(), 0);
    assert_eq!(
        report
            .slots
            .iter()
            .filter(|slot| slot.occupancy == SlotOccupancy::Free)
            .count(),
        6
    );
    let mut recovered = controller(restored(&saved), mock.as_ref().clone());
    recovered.vms = vms.clone();
    recovered.restore().await.unwrap();
    assert_eq!(host.lock().unwrap().deletions.len(), 0);
    assert!(
        matches!(recovered.complete_removal(CompletionAction::Stop).await, Err(Error::HostCommand { program }) if program == "ip")
    );
    let result = recovered.reconcile().await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "ip"));
    assert_eq!(recovered.state.lock().await.pending_networks, [1].into());
    assert!(
        matches!(recovered.complete_removal(CompletionAction::Stop).await, Err(Error::HostCommand { program }) if program == "ip")
    );
    let next = vms.allocate(vm("next", false).report, &[1]).await.unwrap();
    assert_eq!(next.network_index, 2);
    host.lock().unwrap().deletion = Deletion::Succeed;
    recovered.reconcile().await.unwrap();
    assert_eq!(
        recovered.state.lock().await.pending_networks,
        Default::default()
    );
    assert!(
        recovered
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
    assert_eq!(restored(&saved).mode, Mode::Removed);
    assert_eq!(host.lock().unwrap().namespaces, Default::default());
    assert_eq!(
        vms.allocate(vm("reused", false).report, &[])
            .await
            .unwrap()
            .network_index,
        1
    );
}
