//! Checked repository disk attachments with durable bounded capacity leases.

use std::fs::{self, File};
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::cache::{DockerDiskAccess, DockerDiskAttachment, DockerDiskReport};
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use sha2::{Digest, Sha256};

use crate::boundaries::Cache;
use crate::docker_files::{self, Lease};
use crate::docker_prepare::Preparation;
use crate::error::{ErrorResultExt, Result};
use crate::layout::Layout;
use crate::persistence::private_directory;
use crate::process::Commands;
use crate::state::VmRecord;

#[cfg_attr(test,unimock::unimock(api=DockerDisksMock))]
#[async_trait]
pub(crate) trait DockerDisks: Send + Sync {
    fn reserved(&self, vm: &VmRecord) -> Result<bool>;
    async fn prepare(
        &self,
        vm: &VmRecord,
        facts: &JobFacts,
        scope: &JobScope,
        data_bytes: u64,
        spare: u64,
        free_pool: u64,
    ) -> Result<Option<DockerDiskAttachment>>;
    fn release(&self, vm: &VmRecord) -> Result<()>;
    fn report(&self) -> Result<DockerDiskReport>;
    fn reclaim(&self, free_pool: u64) -> Result<u64>;
}

pub(crate) struct LinuxDockers {
    pub(crate) layout: Layout,
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) cache: Arc<dyn Cache>,
}

#[async_trait]
impl DockerDisks for LinuxDockers {
    fn reserved(&self, vm: &VmRecord) -> Result<bool> {
        Ok(docker_files::leases(&self.layout)?
            .iter()
            .any(|(id, _)| *id == vm.local_id))
    }
    async fn prepare(
        &self,
        vm: &VmRecord,
        facts: &JobFacts,
        scope: &JobScope,
        data_bytes: u64,
        spare: u64,
        free_pool: u64,
    ) -> Result<Option<DockerDiskAttachment>> {
        if !facts.sticky_docker || data_bytes == 0 {
            return Ok(None);
        }
        let leases = docker_files::leases(&self.layout)?;
        if let Some((_, lease)) = leases.iter().find(|(id, _)| *id == vm.local_id) {
            return Ok(Some(attachment(
                lease.writer,
                vm.report.requested_size.disk_bytes,
            )));
        }
        let key = hex::encode(Sha256::digest(
            format!("{}:{}", facts.organization_id.0, facts.repository.id.0).as_bytes(),
        ));
        let directory = self.layout.data.join("docker").join(&key);
        private_directory(&directory)?;
        let exclusive = persistent(facts, scope)
            && !leases
                .iter()
                .any(|(_, lease)| lease.writer && lease.repository == key);
        let base = directory.join("base.ext4");
        let path = self.layout.jail_root(vm).join("docker-job.ext4");
        let old_size = match base.metadata() {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error).context("inspect Docker base"),
        };
        let limit = spare
            .min(free_pool)
            .saturating_sub(leases.iter().map(|(_, lease)| lease.bytes).sum());
        let desired = data_bytes
            .saturating_add(vm.report.requested_size.disk_bytes)
            .max(old_size)
            .div_ceil(4096)
            * 4096;
        if desired > limit {
            return Ok(None);
        }
        let lease = Lease {
            repository: key,
            writer: exclusive,
            bytes: desired,
        };
        docker_files::save(&self.layout, vm, &lease)?;
        let result = self
            .prepare_file(
                vm,
                Preparation {
                    base,
                    path,
                    lease,
                    limit,
                    data: data_bytes,
                    free_pool,
                },
            )
            .await;
        match result {
            Ok(()) => Ok(Some(attachment(
                exclusive,
                vm.report.requested_size.disk_bytes,
            ))),
            Err(error) => {
                self.release(vm)?;
                Err(error)
            }
        }
    }

    fn release(&self, vm: &VmRecord) -> Result<()> {
        let directory = self.layout.data.join("docker-leases");
        for path in [
            self.layout.jail_root(vm).join("docker-job.ext4"),
            directory.join(&vm.local_id),
        ] {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("release Docker attachment"),
            }
        }
        match File::open(directory) {
            Ok(directory) => directory.sync_all().context("sync released Docker lease")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("open Docker lease directory"),
        }
        Ok(())
    }

    fn report(&self) -> Result<DockerDiskReport> {
        let bases = docker_files::bases(&self.layout)?;
        let leases = docker_files::leases(&self.layout)?;
        let mut repositories: std::collections::BTreeSet<_> = leases
            .iter()
            .map(|(_, lease)| lease.repository.clone())
            .collect();
        repositories.extend(bases.iter().filter_map(|(_, path, _)| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        }));
        Ok(DockerDiskReport {
            leased_bytes: leases.iter().map(|(_, lease)| lease.bytes).sum(),
            repositories: repositories.len() as u64,
            used_bytes: bases.iter().map(|(_, _, bytes)| bytes).sum(),
        })
    }

    fn reclaim(&self, free_pool: u64) -> Result<u64> {
        let entries = docker_files::unused(&self.layout)?;
        let mut used = entries.iter().map(|(_, _, bytes)| bytes).sum::<u64>();
        for (_, path, bytes) in entries {
            if used <= free_pool {
                break;
            }
            fs::remove_dir_all(path).context("reclaim least-recent unused Docker disk")?;
            used = used.saturating_sub(bytes);
        }
        Ok(used)
    }
}

fn attachment(writer: bool, free_bytes: u64) -> DockerDiskAttachment {
    DockerDiskAttachment {
        free_bytes,
        drive_id: "docker".to_owned(),
        access: if writer {
            DockerDiskAccess::Persistent
        } else {
            DockerDiskAccess::DisposableCopy
        },
    }
}
pub(crate) fn persistent(facts: &JobFacts, scope: &JobScope) -> bool {
    matches!(scope,JobScope::Checked{git_ref,..}if git_ref==&facts.default_ref)
        && matches!(
            facts.event.0.as_str(),
            "push"
                | "workflow_dispatch"
                | "repository_dispatch"
                | "delete"
                | "registry_package"
                | "page_build"
                | "schedule"
        )
        && facts
            .head_repository
            .as_ref()
            .is_none_or(|head| head.id == facts.repository.id)
}
