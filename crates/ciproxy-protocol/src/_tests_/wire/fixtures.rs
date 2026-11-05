//! Fixed public message values with dummy credentials and no ambient input.

use crate::cache::{
    CacheConnection, CacheRestoreCounts, DockerDiskReport, JobCacheReport, JobCacheUsage,
    MachineCacheReport, UnknownJobCount,
};
use crate::identifiers::{
    CommandId, GitRef, ImageId, JobEvent, JobId, LabelName, MachineId, OrganizationId,
    RepositoryId, RunnerId, WorkspaceId,
};
use crate::images::{ImageDownload, ImageMetadata, ImageReport};
use crate::jobs::{JobAssignment, JobFacts, Repository, UnknownJobScopeReason};
use crate::reports::{
    DiskReport, DrainState, HardwareReport, MachineReport, SlotOccupancy, SlotReport,
};
use crate::resources::{GIGABYTE, Slots, VmSize};
use crate::runners::{RunnerReport, RunnerSpec, RunnerState};
use crate::secret::Secret;

pub(crate) const INSTALL_TOKEN: &str = "dummy-install-token";
pub(crate) const CREDENTIAL: &str = "dummy-machine-credential";
pub(crate) const REGISTRATION: &str = "dummy-one-use-registration";
pub(crate) const DOWNLOAD_URL: &str = "https://download.example/image?signature=dummy-image-key";
pub(crate) const CACHE_URL: &str = "http://cache.example/?access=dummy-url-token";
pub(crate) const CACHE_TOKEN: &str = "dummy-cache-token";

pub(crate) fn machine_id() -> MachineId {
    MachineId("machine-1".into())
}
pub(crate) fn workspace_id() -> WorkspaceId {
    WorkspaceId("workspace-1".into())
}
pub(crate) fn organization_id() -> OrganizationId {
    OrganizationId("organization-1".into())
}
pub(crate) fn runner_id() -> RunnerId {
    RunnerId("runner-1".into())
}
pub(crate) fn job_id() -> JobId {
    JobId("job-1".into())
}
pub(crate) fn command_id() -> CommandId {
    CommandId("command-1".into())
}
pub(crate) fn image_id() -> ImageId {
    ImageId("ubuntu-image-1".into())
}

pub(crate) fn size() -> VmSize {
    VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 81_635_856_384,
    }
}

pub(crate) fn runner() -> RunnerSpec {
    RunnerSpec {
        runner_id: runner_id(),
        organization_id: organization_id(),
        label: LabelName("ciproxy".into()),
        size: VmSize {
            disk_bytes: 20 * GIGABYTE,
            ..size()
        },
        registration: Secret::new(REGISTRATION),
    }
}

pub(crate) fn image() -> ImageDownload {
    ImageDownload {
        image_id: image_id(),
        download_url: Secret::new(DOWNLOAD_URL),
        sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
        size_bytes: 12 * GIGABYTE,
        root_disk_bytes: 61 * GIGABYTE,
    }
}

pub(crate) fn cache_connection() -> CacheConnection {
    CacheConnection {
        url: Secret::new(CACHE_URL),
        token: Secret::new(CACHE_TOKEN),
    }
}

pub(crate) fn facts() -> JobFacts {
    JobFacts {
        organization_id: organization_id(),
        repository: Repository {
            id: RepositoryId("repository-1".into()),
            full_name: "example/project".into(),
        },
        assignment: JobAssignment {
            run_id: "123".into(),
            job_id: Some(job_id()),
            runner_id: "456".into(),
        },
        event: JobEvent("pull_request".into()),
        head_repository: Some(Repository {
            id: RepositoryId("repository-2".into()),
            full_name: "contributor/project".into(),
        }),
        default_ref: GitRef("refs/heads/main".into()),
    }
}

pub(crate) fn report() -> MachineReport {
    MachineReport {
        hostname: "build-machine".into(),
        agent_version: "0.1.0".into(),
        hardware: HardwareReport {
            processor: "Example x64 CPU".into(),
            threads: 8,
            memory_bytes: 32 * GIGABYTE,
            kvm: true,
            disks: vec![DiskReport {
                name: "nvme0n1".into(),
                capacity_bytes: 512 * GIGABYTE,
                free_bytes: 300 * GIGABYTE,
            }],
        },
        capacity: Slots(3),
        slots: vec![
            SlotReport {
                index: 0,
                occupancy: SlotOccupancy::Free,
            },
            SlotReport {
                index: 1,
                occupancy: SlotOccupancy::Warm {
                    runner_id: RunnerId("runner-warm".into()),
                },
            },
            SlotReport {
                index: 2,
                occupancy: SlotOccupancy::Job {
                    runner_id: runner_id(),
                    job_id: job_id(),
                },
            },
        ],
        runners: vec![
            RunnerReport {
                runner_id: RunnerId("runner-warm".into()),
                organization_id: OrganizationId("organization-2".into()),
                label: LabelName("ciproxy".into()),
                requested_size: VmSize {
                    vcpu: 2,
                    memory_bytes: 8 * GIGABYTE,
                    disk_bytes: 20 * GIGABYTE,
                },
                size: size(),
                image_id: image_id(),
                state: RunnerState::Warm,
            },
            RunnerReport {
                runner_id: runner_id(),
                organization_id: organization_id(),
                label: LabelName("ciproxy".into()),
                requested_size: VmSize {
                    vcpu: 2,
                    memory_bytes: 8 * GIGABYTE,
                    disk_bytes: 20 * GIGABYTE,
                },
                size: size(),
                image_id: ImageId("ubuntu-image-0".into()),
                state: RunnerState::Running { job_id: job_id() },
            },
        ],
        drain: DrainState::Active,
        cache: MachineCacheReport {
            used_bytes: 20 * GIGABYTE,
            capacity_bytes: 100 * GIGABYTE,
            restores_last_7_days: 100,
            hits_last_7_days: 75,
            unknown_jobs: vec![UnknownJobCount {
                reason: UnknownJobScopeReason::NoWriteScope,
                jobs: 1,
            }],
            jobs: vec![
                JobCacheReport {
                    job_id: job_id(),
                    usage: JobCacheUsage::MachineHandled(CacheRestoreCounts {
                        local_entries: 3,
                        github_entries: 0,
                        restored_bytes: 2 * GIGABYTE,
                    }),
                },
                JobCacheReport {
                    job_id: JobId("job-2".into()),
                    usage: JobCacheUsage::MachineHandled(CacheRestoreCounts {
                        local_entries: 1,
                        github_entries: 1,
                        restored_bytes: GIGABYTE,
                    }),
                },
                JobCacheReport {
                    job_id: JobId("job-3".into()),
                    usage: JobCacheUsage::Github,
                },
                JobCacheReport {
                    job_id: JobId("job-4".into()),
                    usage: JobCacheUsage::MachineHandled(CacheRestoreCounts {
                        local_entries: 0,
                        github_entries: 2,
                        restored_bytes: 3 * GIGABYTE,
                    }),
                },
            ],
        },
        docker_disks: DockerDiskReport {
            repositories: 2,
            used_bytes: 15 * GIGABYTE,
        },
        images: ImageReport {
            available: vec![
                ImageMetadata {
                    image_id: ImageId("ubuntu-image-0".into()),
                    root_disk_bytes: 59 * GIGABYTE,
                },
                ImageMetadata {
                    image_id: image_id(),
                    root_disk_bytes: 61 * GIGABYTE,
                },
            ],
            newest: Some(image_id()),
        },
    }
}
