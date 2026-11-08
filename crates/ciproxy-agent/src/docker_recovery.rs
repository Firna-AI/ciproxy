//! Failed attachment recovery retains capacity and excludes live source writers.

use std::fs::{self, File};
use std::path::Path;

use crate::docker::LinuxDockers;
use crate::docker_files::{self, AttachmentPhase, Lease};
use crate::error::{Error, ErrorResultExt, Result};
use crate::state::VmRecord;

impl LinuxDockers {
    pub(crate) fn fail_attachment(&self, vm: &VmRecord) -> Result<bool> {
        let Some(mut lease) = docker_files::load(&self.layout, vm)? else {
            return Ok(false);
        };
        lease.phase = match lease.phase {
            AttachmentPhase::Preparing => AttachmentPhase::Abandon,
            AttachmentPhase::Offered => AttachmentPhase::Discard,
            AttachmentPhase::Detached => return Ok(false),
            phase => phase,
        };
        docker_files::save(&self.layout, vm, &lease)?;
        Ok(true)
    }

    /// The caller must first close the VM drive or stop the VM completely.
    pub(crate) async fn discard_attachment(&self, vm: &VmRecord) -> Result<()> {
        let Some(mut lease) = docker_files::load(&self.layout, vm)? else {
            return Ok(());
        };
        if !matches!(
            lease.phase,
            AttachmentPhase::Discard | AttachmentPhase::Abandon
        ) {
            return Ok(());
        }
        if lease.phase == AttachmentPhase::Discard {
            if lease.writer {
                remove(&self.base(&lease))?;
            } else {
                self.discard_invalid_source(vm, &lease).await?;
            }
        }
        remove(&self.layout.jail_root(vm).join("docker-job.ext4"))?;
        lease.writer = false;
        lease.phase = AttachmentPhase::Detached;
        docker_files::save(&self.layout, vm, &lease)
    }

    /// Copies can invalidate only an idle source that also fails its own check.
    pub(crate) async fn discard_invalid_source(&self, vm: &VmRecord, lease: &Lease) -> Result<()> {
        if docker_files::leases(&self.layout)?
            .iter()
            .any(|(id, active)| {
                id != &vm.local_id && active.writer && active.repository == lease.repository
            })
        {
            return Ok(());
        }
        let base = self.base(lease);
        if lease.writer {
            return remove(&base);
        }
        match fs::metadata(&base) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error).context("inspect failed Docker source"),
        }
        match self
            .commands
            .check_filesystem(&base.to_string_lossy())
            .await
        {
            Ok(()) => Ok(()),
            Err(Error::DockerFilesystem) => remove(&base),
            Err(error) => Err(error),
        }
    }

    fn base(&self, lease: &Lease) -> std::path::PathBuf {
        self.layout
            .data
            .join("docker")
            .join(&lease.repository)
            .join("base.ext4")
    }
}

fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("discard failed Docker disk"),
    }
    if let Some(parent) = path.parent() {
        match File::open(parent) {
            Ok(directory) => directory.sync_all().context("sync discarded Docker disk")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("open discarded Docker directory"),
        }
    }
    Ok(())
}
