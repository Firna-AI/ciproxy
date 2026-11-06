//! Fixed complete reports, wire versions, and handshake builders.

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::cache::{CacheRestoreCounts, JobCacheReport, JobCacheUsage};
use ciproxy_protocol::identifiers::{
    ImageId, JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::reports::{DiskReport, MachineReport, SlotOccupancy, SlotReport};
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};
use ciproxy_protocol::runners::{RunnerReport, RunnerState};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use uuid::Uuid;

pub(super) fn version(number: u16) -> ProtocolVersion {
    serde_json::from_str(&number.to_string()).unwrap()
}

pub(super) fn report() -> MachineReport {
    let message: AgentMessage =
        serde_json::from_str(include_str!("../../src/_tests_/unsupported_hello.json")).unwrap();
    let AgentMessage::Hello { mut report, .. } = message else {
        panic!("report fixture")
    };
    report.hostname = "build-one".into();
    report.hardware.threads = 8;
    report.hardware.memory_bytes = 32 * GIGABYTE;
    report.hardware.disks = vec![DiskReport {
        name: "/var/lib/ciproxy".into(),
        capacity_bytes: 512 * GIGABYTE,
        free_bytes: 123 * GIGABYTE,
    }];
    report.slots = vec![
        SlotReport {
            index: 0,
            occupancy: SlotOccupancy::Warm {
                runner_id: RunnerId("warm".into()),
            },
        },
        SlotReport {
            index: 1,
            occupancy: SlotOccupancy::Job {
                runner_id: RunnerId("busy".into()),
                job_id: JobId("job-9".into()),
            },
        },
        SlotReport {
            index: 2,
            occupancy: SlotOccupancy::Free,
        },
    ];
    report.cache.used_bytes = 4 * GIGABYTE;
    report.cache.capacity_bytes = 117_200_774_656;
    report.cache.restores_last_7_days = 19;
    report.cache.hits_last_7_days = 11;
    report.cache.jobs = vec![JobCacheReport {
        organization_id: OrganizationId("org-2".into()),
        repository_id: ciproxy_protocol::identifiers::RepositoryId("repo-9".into()),
        assignment: ciproxy_protocol::jobs::JobAssignment {
            run_id: "run-9".into(),
            job_id: Some(JobId("job-9".into())),
            runner_id: "github-runner-9".into(),
        },
        job_id: JobId("job-9".into()),
        usage: JobCacheUsage::MachineHandled(CacheRestoreCounts {
            local_entries: 2,
            github_entries: 3,
            restored_bytes: 72_000,
        }),
    }];
    report.docker_disks.repositories = 3;
    report.docker_disks.used_bytes = 8 * GIGABYTE;
    report.images.available = vec![ImageMetadata {
        image_id: ImageId("image-4".into()),
        root_disk_bytes: 60_263_759_872,
    }];
    report.images.newest = Some(ImageId("image-4".into()));
    report.capacity = Slots(3);
    report.job_disk_bytes = 279 * GIGABYTE;
    report.runners = vec![
        RunnerReport {
            slot_indices: vec![1],
            runner_id: RunnerId("busy".into()),
            organization_id: OrganizationId("org-2".into()),
            label: LabelName("ciproxy".into()),
            requested_size: VmSize {
                vcpu: 2,
                memory_bytes: 8 * GIGABYTE,
                disk_bytes: 20 * GIGABYTE,
            },
            size: VmSize {
                vcpu: 2,
                memory_bytes: 8 * GIGABYTE,
                disk_bytes: 80_899_612_672,
            },
            image_id: ImageId("image-3".into()),
            state: RunnerState::Running {
                job_id: JobId("job-9".into()),
            },
        },
        RunnerReport {
            slot_indices: vec![0],
            runner_id: RunnerId("warm".into()),
            organization_id: OrganizationId("org-1".into()),
            label: LabelName("ciproxy".into()),
            requested_size: VmSize {
                vcpu: 2,
                memory_bytes: 8 * GIGABYTE,
                disk_bytes: 20 * GIGABYTE,
            },
            size: VmSize {
                vcpu: 2,
                memory_bytes: 8 * GIGABYTE,
                disk_bytes: 80_899_612_672,
            },
            image_id: ImageId("image-4".into()),
            state: RunnerState::Warm,
        },
    ];
    report
}

pub(super) fn register(token: Secret, report: MachineReport) -> AgentMessage {
    AgentMessage::Register {
        protocol_version: ProtocolVersion::CURRENT,
        install_token: token,
        report,
    }
}

pub(super) fn hello(
    workspace: Uuid,
    machine: MachineId,
    credential: Secret,
    number: u16,
    report: MachineReport,
) -> AgentMessage {
    AgentMessage::Hello {
        protocol_version: version(number),
        workspace_id: WorkspaceId(workspace.to_string()),
        machine_id: machine,
        credential,
        report,
    }
}
