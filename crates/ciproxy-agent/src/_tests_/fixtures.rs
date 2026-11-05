//! Owned test data and Unimock wiring. No host resources are used.

use std::sync::Arc;

use ciproxy_protocol::agent::{AgentMessage, CommandOutcome};
use ciproxy_protocol::identifiers::{
    CommandId, ImageId, JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::images::ImageReport;
use ciproxy_protocol::reports::{DiskReport, HardwareReport, MachineReport};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_protocol::runners::{RunnerReport, RunnerSpec, RunnerState};
use ciproxy_protocol::secret::Secret;
use tokio::sync::Mutex;
use unimock::{Clause, MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, ImagesMock};
use crate::controller::{Agent, Controller};
use crate::image_registry::{ImageLease, ImageLeaseMock};
use crate::reporting::empty_cache;
use crate::state::{Identity, MachineState, VmRecord};

pub(super) fn identity() -> Identity {
    Identity {
        workspace_id: WorkspaceId("workspace-1".to_owned()),
        machine_id: MachineId("machine-1".to_owned()),
        credential: Secret::new("machine-secret"),
    }
}

pub(super) fn vm(name: &str, running: bool) -> VmRecord {
    VmRecord {
        local_id: format!("local-{name}"),
        network_index: 1,
        report: RunnerReport {
            runner_id: RunnerId(name.to_owned()),
            organization_id: OrganizationId("org-1".to_owned()),
            label: LabelName("ciproxy".to_owned()),
            size: size(),
            image_id: ImageId("image-1".to_owned()),
            state: if running {
                RunnerState::Running {
                    job_id: JobId("job-1".to_owned()),
                }
            } else {
                RunnerState::Warm
            },
        },
    }
}

pub(super) fn size() -> VmSize {
    VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 60 * GIGABYTE,
    }
}

pub(super) fn spec() -> RunnerSpec {
    RunnerSpec {
        runner_id: RunnerId("runner-1".to_owned()),
        organization_id: OrganizationId("org-1".to_owned()),
        label: LabelName("ciproxy".to_owned()),
        size: size(),
        registration: Secret::new("one-use-registration"),
    }
}

pub(super) fn hardware() -> HardwareReport {
    HardwareReport {
        processor: "Fixture CPU".to_owned(),
        threads: 16,
        memory_bytes: 64 * GIGABYTE,
        kvm: true,
        disks: vec![DiskReport {
            name: "data".to_owned(),
            capacity_bytes: 1000 * GIGABYTE,
            free_bytes: 900 * GIGABYTE,
        }],
    }
}

pub(super) fn report_dependencies() -> impl Clause {
    report_dependencies_for(hardware())
}

pub(super) fn report_dependencies_for(hardware: HardwareReport) -> impl Clause {
    (
        HostMock::hardware
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(hardware.clone()))),
        HostMock::hostname
            .each_call(matching!())
            .answers(&|_| Ok("fixture-machine".to_owned())),
        CacheMock::report
            .each_call(matching!())
            .answers(&|_| Ok(empty_cache())),
        ImagesMock::report.each_call(matching!()).answers(&|_| {
            Ok(ImageReport {
                available: vec![ImageId("image-1".to_owned())],
                newest: Some(ImageId("image-1".to_owned())),
            })
        }),
    )
}

pub(super) fn controller(state: MachineState, mock: Unimock) -> Controller {
    let mock = Arc::new(mock);
    Controller {
        state: Mutex::new(state),
        store: mock.clone(),
        host: mock.clone(),
        vms: mock.clone(),
        images: mock.clone(),
        cache: mock,
    }
}

pub(super) fn applied() -> Option<AgentMessage> {
    Some(AgentMessage::CommandResult {
        command_id: CommandId("command-1".to_owned()),
        outcome: CommandOutcome::Applied,
    })
}

pub(super) async fn report() -> MachineReport {
    controller(MachineState::default(), Unimock::new(report_dependencies()))
        .report()
        .await
        .unwrap()
}

pub(super) fn start_image() -> impl Clause {
    ImagesMock::hold_newest
        .each_call(matching!())
        .answers(&|_| Ok(lease("image-1")))
}

pub(super) fn lease(id: &str) -> Arc<dyn ImageLease> {
    Arc::new(Unimock::new(
        ImageLeaseMock::image_id
            .some_call(matching!())
            .returns(ImageId(id.to_owned())),
    ))
}
