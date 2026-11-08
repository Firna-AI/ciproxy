//! Install the released host cache executable and its mount-bound systemd unit.

use std::sync::Arc;

use async_trait::async_trait;
use cli_updater::{UpdateCheck, UpdateRequest, UpgradeOutcome};

use crate::cache_install::{CacheInstallFiles, CacheRelease};
use crate::cache_network::CacheStart;
use crate::cache_unit::CacheUnit;
use crate::error::Result;
use crate::process::{Commands, arguments};

#[async_trait]
pub(crate) trait CacheService: Send + Sync {
    async fn install(&self) -> Result<()>;
}

pub(crate) struct ReleasedCache {
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) files: Arc<dyn CacheInstallFiles>,
    pub(crate) release: Arc<dyn CacheRelease>,
    pub(crate) starter: Arc<dyn CacheStart>,
    pub(crate) unit: Arc<dyn CacheUnit>,
}

#[async_trait]
impl CacheService for ReleasedCache {
    async fn install(&self) -> Result<()> {
        let installed = self.files.prepare()?;
        let unit_changed = self.unit.changed();
        self.unit.install()?;
        self.commands
            .run("systemctl", &arguments(&["daemon-reload"]), &[])
            .await?;
        let running_protected = self.running_protected().await;
        let activation_required = unit_changed || !running_protected;
        if activation_required {
            self.commands
                .run(
                    "systemctl",
                    &arguments(&["stop", "ciproxy-cache.service"]),
                    &[],
                )
                .await?;
        }
        self.starter.repair().await?;
        if activation_required
            && installed.binary_exists
            && let Err(error) = self.starter.start(true).await
        {
            tracing::warn!(%error, "protected cache start unavailable; checked release repair continues");
        }
        let restart = installed.version.trim() != env!("CARGO_PKG_VERSION")
            || !installed.binary_exists
            || activation_required;
        if restart {
            if !installed.binary_exists {
                self.files.stage_binary()?;
            }
            let outcome = self
                .release
                .upgrade(UpdateRequest {
                    check: UpdateCheck {
                        binary: "ciproxy-cache-sidecar".to_owned(),
                        current_version: installed.version.trim().to_owned(),
                        target: "x86_64-unknown-linux-gnu".to_owned(),
                    },
                    current_exe: installed.binary,
                    force: true,
                })
                .await?;
            if let UpgradeOutcome::Installed(asset) = outcome {
                self.files.record_version(&asset.latest.version)?;
            }
        }
        self.starter.start(restart).await?;
        Ok(())
    }
}

impl ReleasedCache {
    async fn running_protected(&self) -> bool {
        let Ok(pid) = self
            .commands
            .run(
                "systemctl",
                &arguments(&[
                    "show",
                    "--property=MainPID",
                    "--value",
                    "ciproxy-cache.service",
                ]),
                &[],
            )
            .await
        else {
            return false;
        };
        let Ok(pid) = std::str::from_utf8(&pid) else {
            return false;
        };
        let Ok(pid) = pid.trim().parse::<u32>() else {
            return false;
        };
        pid != 0 && self.unit.running_matches(pid)
    }
}

#[cfg(test)]
#[path = "_tests_/cache_service_tests.rs"]
mod cache_service_tests;
