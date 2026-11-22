//! `launchctl` for the VM helper jobs in the system domain.

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::{Duration, Instant, sleep};

use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::launchd::{launchctl, loaded_ids};

#[async_trait]
pub(crate) trait Launchd: Send + Sync {
    /// Load and start the helper of a VM from its launchd file.
    async fn bootstrap_vm(&self, local_id: &str) -> Result<()>;
    /// Unload a loaded helper. launchd sends SIGTERM, so the helper stops
    /// its VM, and SIGKILL after the exit timeout.
    async fn bootout_vm(&self, local_id: &str) -> Result<()>;
    async fn loaded(&self, local_id: &str) -> Result<bool>;
    /// The local IDs of every loaded helper, also of helpers with no record.
    async fn loaded_vms(&self) -> Result<Vec<String>>;
}

/// The helper's `ExitTimeOut` of 30 s, then SIGKILL, and a margin.
const UNLOAD_WAIT: Duration = Duration::from_secs(40);

pub(crate) struct Launchctl;

#[async_trait]
impl Launchd for Launchctl {
    async fn bootstrap_vm(&self, local_id: &str) -> Result<()> {
        run(&launchctl::bootstrap_vm(local_id)).await.map(drop)
    }

    /// `launchctl bootout` can return once it has sent SIGTERM. launchd
    /// keeps the job until the helper has stopped its VM and exited. So
    /// wait until the job is gone, up to the exit timeout and a margin.
    /// A later start then never sees a stale job.
    async fn bootout_vm(&self, local_id: &str) -> Result<()> {
        if !self.loaded(local_id).await? {
            return Ok(());
        }
        if let Err(error) = run(&launchctl::bootout_vm(local_id)).await {
            // The job can end between the check and the bootout.
            if self.loaded(local_id).await? {
                return Err(error);
            }
        }
        let deadline = Instant::now() + UNLOAD_WAIT;
        while self.loaded(local_id).await? {
            if Instant::now() >= deadline {
                tracing::warn!(local_id, "the VM helper job did not unload");
                return Err(Error::HostCommand {
                    program: "launchctl".to_owned(),
                });
            }
            sleep(Duration::from_millis(250)).await;
        }
        Ok(())
    }

    async fn loaded(&self, local_id: &str) -> Result<bool> {
        let status = Command::new("launchctl")
            .args(launchctl::print_vm(local_id))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .context("inspect VM helper job")?;
        Ok(status.success())
    }

    async fn loaded_vms(&self) -> Result<Vec<String>> {
        let output = run(&launchctl::list()).await?;
        Ok(loaded_ids(&String::from_utf8_lossy(&output)))
    }
}

async fn run(args: &[String]) -> Result<Vec<u8>> {
    let output = Command::new("launchctl")
        .args(args)
        .output()
        .await
        .context("run launchctl")?;
    if !output.status.success() {
        tracing::warn!(
            status = ?output.status.code(),
            stderr = %String::from_utf8_lossy(&output.stderr).trim(),
            "launchctl failed"
        );
        return Err(Error::HostCommand {
            program: "launchctl".to_owned(),
        });
    }
    Ok(output.stdout)
}
