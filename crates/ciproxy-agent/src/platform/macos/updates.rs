//! Hourly agent updates on a Mac. The installer moves the old binary away
//! and writes the new one as a new file, so macOS never sees a signed
//! binary change in place. VM helpers keep the old binary until they end.

use std::sync::Arc;

use async_trait::async_trait;
use cli_updater::{TarGzBinaryInstaller, UpdateCheck, UpdateRequest, Updater, UpgradeOutcome};

use crate::boundaries::Updates;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::launchd::launchctl;
use crate::release_client::SecureReleases;
use crate::updates_checked::CheckedInstaller;

pub(crate) struct MacUpdates {
    pub(crate) release_server: String,
}

#[async_trait]
impl Updates for MacUpdates {
    async fn check(&self) -> Result<bool> {
        let updater = Updater::new(
            Arc::new(
                SecureReleases::new(self.release_server.clone())
                    .context("configure CLI updater")?,
            ),
            Arc::new(CheckedInstaller {
                installer: Arc::new(TarGzBinaryInstaller::new()),
            }),
        );
        let outcome = updater
            .upgrade(UpdateRequest {
                check: UpdateCheck {
                    binary: "ciproxy-agent".to_owned(),
                    current_version: env!("CARGO_PKG_VERSION").to_owned(),
                    target: "aarch64-apple-darwin".to_owned(),
                },
                current_exe: std::env::current_exe().context("locate installed agent")?,
                force: false,
            })
            .await
            .context("update agent")?;
        Ok(matches!(outcome, UpgradeOutcome::Installed(_)))
    }

    /// Restart the agent job. Running VM helpers are other jobs.
    async fn restart(&self) -> Result<()> {
        let status = tokio::process::Command::new("launchctl")
            .args(launchctl::restart_agent())
            .status()
            .await
            .context("restart the agent")?;
        if !status.success() {
            return Err(Error::HostCommand {
                program: "launchctl".to_owned(),
            });
        }
        Ok(())
    }
}
