//! Owned test data and Unimock wiring. No host resources are used.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ciproxy_agent::root_sizing::{DEFAULT_IMAGE_ROOT_BYTES, RootSizing};
use ciproxy_protocol::agent::{AgentMessage, CommandOutcome};
use ciproxy_protocol::guest::{AgentToGuest, JobDecision};
use ciproxy_protocol::identifiers::{
    CommandId, ImageId, JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::images::{ImageMetadata, ImageReport};
use ciproxy_protocol::reports::{DiskReport, HardwareReport, MachineReport};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_protocol::runners::{RunnerReport, RunnerSpec, RunnerState};
use ciproxy_protocol::secret::Secret;
use tokio::sync::Mutex;
use unimock::{Clause, MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, ImagesMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::controller::{Agent, Controller};
use crate::image_registry::{ImageLease, ImageLeaseMock};
use crate::reporting::empty_cache;
use crate::state::{Identity, MachineState, VmRecord};
use crate::storage_sizing::DEFAULT_IMAGE_BYTES;

pub(super) fn identity() -> Identity {
    Identity {
        workspace_id: WorkspaceId("workspace-1".to_owned()),
        machine_id: MachineId("machine-1".to_owned()),
        credential: Secret::new("machine-secret"),
    }
}

pub(super) fn vm(name: &str, running: bool) -> VmRecord {
    VmRecord {
        preparation: crate::state::PreparationState::Waiting,
        cache_authority: CacheAuthority::Waiting,
        job_facts: None,
        local_id: format!("local-{name}"),
        network_index: 1,
        report: RunnerReport {
            configuration_lost: false,
            slot_indices: if name == "idle" { vec![1] } else { vec![0] },
            runner_id: RunnerId(name.to_owned()),
            organization_id: OrganizationId("org-1".to_owned()),
            label: LabelName("ciproxy".to_owned()),
            requested_size: requested_size(),
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
    RootSizing::default().vm_size(requested_size()).unwrap()
}

pub(super) fn spec() -> RunnerSpec {
    RunnerSpec {
        slot_indices: vec![0],
        runner_id: RunnerId("runner-1".to_owned()),
        organization_id: OrganizationId("org-1".to_owned()),
        label: LabelName("ciproxy".to_owned()),
        size: requested_size(),
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
                available: vec![metadata("image-1", DEFAULT_IMAGE_ROOT_BYTES)],
                newest: Some(ImageId("image-1".to_owned())),
            })
        }),
    )
}

pub(super) fn controller(state: MachineState, mock: Unimock) -> Controller {
    let mock = Arc::new(mock);
    Controller {
        allocation: Mutex::new(()),
        network_cleanup: Mutex::new(()),
        control_connected: AtomicBool::new(true),
        state: Mutex::new(state),
        store: mock.clone(),
        host: mock.clone(),
        vms: mock.clone(),
        images: mock.clone(),
        cache: mock,
        image_budget: DEFAULT_IMAGE_BYTES,
        dockers: None,
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
        .answers(&|_| Ok(lease(metadata("image-1", DEFAULT_IMAGE_ROOT_BYTES))))
}

pub(super) fn lease(image: ImageMetadata) -> Arc<dyn ImageLease> {
    Arc::new(Unimock::new(
        ImageLeaseMock::metadata
            .some_call(matching!())
            .returns(image),
    ))
}

pub(super) fn requested_size() -> VmSize {
    VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 20 * GIGABYTE,
    }
}

pub(super) fn metadata(id: &str, root_disk_bytes: u64) -> ImageMetadata {
    ImageMetadata {
        image_id: ImageId(id.to_owned()),
        root_disk_bytes,
    }
}

pub(super) fn trusted_facts() -> ciproxy_protocol::jobs::JobFacts {
    serde_json::from_value(serde_json::json!({"organization_id":"org-1","repository":{"id":"repo-1","full_name":"first/project"},"assignment":{"run_id":"run-1","job_id":"job-1","runner_id":"github-runner-1"},"event":"push","head_repository":null,"default_ref":"refs/heads/main","sticky_docker":true})).unwrap()
}

pub(super) fn github_decision_sent() -> impl Clause {
    VmsMock::send
        .each_call(matching!(_, _))
        .answers(&|_, record, message| {
            assert_eq!(record.cache_authority, CacheAuthority::GitHubOnly);
            assert_eq!(
                message,
                &AgentToGuest::PrepareJob {
                    job_id: JobId("job-1".into()),
                    decision: JobDecision::GitHubOnly
                }
            );
            Ok(())
        })
}
