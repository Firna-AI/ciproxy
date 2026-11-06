//! Unique temporary layouts and typed inputs for Docker filesystem regressions.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::identifiers::{
    GitRef, ImageId, JobEvent, LabelName, OrganizationId, RepositoryId, RunnerId,
};
use ciproxy_protocol::images::ImageMetadata;
use ciproxy_protocol::jobs::{JobAssignment, JobFacts, JobScope, Repository};
use ciproxy_protocol::resources::VmSize;
use ciproxy_protocol::runners::{RunnerReport, RunnerState};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::CacheMock;
use crate::docker::{DockerDisks, LinuxDockers};
use crate::docker_files;
use crate::error::Error;
use crate::layout::Layout;
use crate::state::{PreparationState, VmRecord};

pub(super) struct TemporaryLayout {
    pub(super) layout: Layout,
}

impl TemporaryLayout {
    pub(super) fn new() -> Self {
        let data = std::env::temp_dir().join(format!("ciproxy-docker-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&data).unwrap();
        Self {
            layout: Layout {
                data,
                firecracker: PathBuf::from("/usr/bin/firecracker"),
                jailer: PathBuf::from("/usr/bin/jailer"),
            },
        }
    }

    pub(super) fn adapter(&self, mock: Unimock) -> LinuxDockers {
        let mock = Arc::new(mock);
        LinuxDockers {
            layout: self.layout.clone(),
            commands: mock.clone(),
            cache: mock,
        }
    }

    pub(super) fn lease_path(&self, vm: &VmRecord) -> PathBuf {
        self.layout.data.join("docker-leases").join(&vm.local_id)
    }

    pub(super) async fn repository(&self) -> String {
        let repository = Arc::new(Mutex::new(None));
        let captured = repository.clone();
        let layout = self.layout.clone();
        let adapter = self.adapter(Unimock::new(
            CacheMock::budget
                .some_call(matching!(_))
                .answers_arc(Arc::new(move |_, _| {
                    let leases = docker_files::leases(&layout).unwrap();
                    *captured.lock().unwrap() = Some(leases[0].1.repository.clone());
                    Err(Error::CacheUnavailable)
                })),
        ));
        let vm = vm("repository-key");
        assert!(
            adapter
                .prepare(&vm, &facts(), &scope(), 4096, 65536, 65536)
                .await
                .is_err()
        );
        repository.lock().unwrap().take().unwrap()
    }
}

impl Drop for TemporaryLayout {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.layout.data).unwrap();
    }
}

pub(super) fn vm(name: &str) -> VmRecord {
    let size = VmSize {
        vcpu: 1,
        memory_bytes: 4096,
        disk_bytes: 4096,
    };
    VmRecord {
        preparation: PreparationState::Started,
        cache_permissions: None,
        job_facts: None,
        local_id: format!("local-{name}"),
        network_index: 1,
        report: RunnerReport {
            runner_id: RunnerId(name.to_owned()),
            organization_id: OrganizationId("org-1".to_owned()),
            label: LabelName("ciproxy".to_owned()),
            requested_size: size,
            size,
            slot_indices: vec![0],
            image_id: ImageId("image-1".to_owned()),
            state: RunnerState::Warm,
            configuration_lost: false,
        },
    }
}

pub(super) fn facts() -> JobFacts {
    JobFacts {
        organization_id: OrganizationId("org-1".to_owned()),
        repository: Repository {
            id: RepositoryId("repo-1".to_owned()),
            full_name: "first/project".to_owned(),
        },
        assignment: JobAssignment {
            run_id: "run-1".to_owned(),
            job_id: None,
            runner_id: "github-runner-1".to_owned(),
        },
        event: JobEvent("push".to_owned()),
        head_repository: None,
        default_ref: GitRef("refs/heads/main".to_owned()),
        sticky_docker: true,
    }
}

pub(super) fn scope() -> JobScope {
    JobScope::Checked {
        git_ref: facts().default_ref,
        read_refs: Vec::new(),
    }
}

pub(super) fn metadata(id: &str, root_disk_bytes: u64) -> ImageMetadata {
    ImageMetadata {
        image_id: ImageId(id.to_owned()),
        root_disk_bytes,
    }
}
