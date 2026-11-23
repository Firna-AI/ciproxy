//! A Mac starts a VM only when its area volume has the VM's disk as real
//! free space, also when the job disk count has room.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::GIGABYTE;
use ciproxy_protocol::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, ImagesMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{
    applied, controller, hardware, lease, metadata, report_dependencies_for, spec, vm,
};
use crate::state::MachineState;

/// The standard macOS release: a 100 GB disk with 21.8 GB free for a job.
fn macos_image() -> ImageMetadata {
    ImageMetadata {
        free_job_bytes: Some(21_800_000_000),
        ..metadata("image-1", 100 * GIGABYTE)
    }
}

/// The runner keeps the build's native platform, so the command passes its
/// platform check. The disk rule follows the hardware report, a Mac here.
fn start() -> ControlMessage {
    ControlMessage::StartRunner {
        command_id: CommandId("command-1".to_owned()),
        runner: spec(),
    }
}

/// A Mac with 300 GB of area: its job disk count has room for the VM, and
/// its real free space is `free_bytes`.
fn mac_agent(free_bytes: u64, rest: impl unimock::Clause) -> crate::controller::Controller {
    let mut mac = hardware();
    mac.platform = Platform::MacosArm64;
    mac.disks[0].capacity_bytes = 300 * GIGABYTE;
    mac.disks[0].free_bytes = free_bytes;
    let mock = Unimock::new((
        report_dependencies_for(mac),
        ImagesMock::hold_newest
            .each_call(matching!())
            .answers(&|_| Ok(lease(macos_image()))),
        rest,
    ));
    controller(MachineState::default(), mock)
}

/// A 20 GB label holds 24 GB, so 23 GB of real free space refuses it
/// before any VM is allocated.
#[tokio::test]
async fn a_mac_refuses_a_vm_that_its_real_free_space_cannot_hold() {
    let agent = mac_agent(23 * GIGABYTE, ());

    assert_eq!(
        agent.control(start()).await.unwrap(),
        Some(AgentMessage::CommandResult {
            command_id: CommandId("command-1".to_owned()),
            outcome: CommandOutcome::Failed {
                reason: CommandFailure::RunnerStart
            },
        })
    );
    assert!(agent.state.lock().await.runners.is_empty());
}

/// With 24 GB of real free space, the same start holds the label disk and
/// the margin, not the 100 GB image disk.
#[tokio::test]
async fn a_mac_starts_a_vm_that_holds_its_label_disk_and_a_margin() {
    let agent = mac_agent(
        24 * GIGABYTE,
        (
            VmsMock::allocate
                .some_call(matching!((report, _) if report.state == RunnerState::Starting))
                .answers(&|_, report, _| {
                    let mut allocated = vm("runner-1", false);
                    allocated.report = report;
                    Ok(allocated)
                }),
            StoreMock::save
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            CacheMock::budget
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            VmsMock::start.some_call(matching!(_, _)).returns(Ok(())),
        ),
    );

    assert_eq!(agent.control(start()).await.unwrap(), applied());
    let state = agent.state.lock().await;
    assert_eq!(state.runners.len(), 1);
    assert_eq!(state.runners[0].report.size.disk_bytes, 24 * GIGABYTE);
}
