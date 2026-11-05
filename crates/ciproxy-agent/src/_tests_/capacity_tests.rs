//! Machines connect with zero slots when KVM or host capacity is absent.

use unimock::Unimock;

use crate::controller::Agent;
use crate::fixtures::{controller, hardware, report_dependencies_for};
use crate::state::MachineState;

#[tokio::test]
async fn a_machine_without_kvm_or_slot_capacity_reports_zero_slots() {
    let mut no_kvm = hardware();
    no_kvm.kvm = false;
    let mut too_small = hardware();
    too_small.memory_bytes = 1_000_000_000;
    for hardware in [no_kvm, too_small] {
        let agent = controller(
            MachineState::default(),
            Unimock::new(report_dependencies_for(hardware)),
        );
        let report = agent.report().await.unwrap();
        assert_eq!(report.capacity.0, 0);
        assert_eq!(report.slots, Vec::new());
    }
}
