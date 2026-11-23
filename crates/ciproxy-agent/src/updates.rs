//! Hourly CLI upgrades replace only the agent binary and restart its own unit.

use std::sync::Arc;

use async_trait::async_trait;
use cli_updater::{TarGzBinaryInstaller, UpdateCheck, UpdateRequest, Updater, UpgradeOutcome};

use crate::boundaries::Updates;
use crate::cache_install::{CheckedCacheRelease, LinuxCacheInstallFiles};
use crate::cache_network::GuardedCacheStart;
use crate::cache_service::{CacheService, ReleasedCache};
use crate::cache_unit::{InstalledCacheUnit, LinuxCacheUnitFiles};
use crate::error::{ErrorResultExt, Result};
use crate::process::{Commands, arguments};
use crate::release_client::SecureReleases;
use crate::updates_checked::CheckedInstaller;

pub(crate) struct AgentUpdates {
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) release_server: String,
}

#[async_trait]
impl Updates for AgentUpdates {
    async fn check(&self) -> Result<bool> {
        let cache: Arc<dyn CacheService> = Arc::new(ReleasedCache {
            unit: Arc::new(InstalledCacheUnit {
                files: Arc::new(LinuxCacheUnitFiles),
            }),
            starter: Arc::new(GuardedCacheStart {
                commands: self.commands.clone(),
            }),
            commands: self.commands.clone(),
            files: Arc::new(LinuxCacheInstallFiles),
            release: Arc::new(CheckedCacheRelease {
                release_server: self.release_server.clone(),
            }),
        });
        if let Err(error) = cache.install().await {
            tracing::warn!(%error, "machine cache update will retry");
        }
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
                    target: "x86_64-unknown-linux-gnu".to_owned(),
                },
                current_exe: std::env::current_exe().context("locate installed agent")?,
                force: false,
            })
            .await
            .context("update agent")?;
        Ok(matches!(outcome, UpgradeOutcome::Installed(_)))
    }

    async fn restart(&self) -> Result<()> {
        self.commands
            .run(
                "systemctl",
                &arguments(&["--no-block", "try-restart", "ciproxy-agent.service"]),
                &[],
            )
            .await?;
        Ok(())
    }
}
