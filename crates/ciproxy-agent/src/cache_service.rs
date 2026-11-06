//! Install the released host cache executable and its mount-bound systemd unit.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::secret::Secret;
use cli_updater::{BinaryInstaller, TarGzBinaryInstaller, UpdateCheck, UpdateRequest, Updater};

use crate::cache_network::CacheStart;
use crate::error::{ErrorResultExt, Result};
use crate::persistence::{atomic_private_write, private_directory};
use crate::process::{Commands, arguments};
use crate::release_client::SecureReleases;
use crate::updates::CheckedInstaller;

const BINARY: &str = "/usr/local/bin/ciproxy-cache-sidecar";
const VERSION: &str = "/var/lib/ciproxy/cache-sidecar-version";
const UNIT: &str = "[Unit]\nDescription=ciproxy machine cache\nRequires=var-lib-ciproxy-area.mount\nAfter=var-lib-ciproxy-area.mount network-online.target\nRequiresMountsFor=/var/lib/ciproxy/area/cache\n\n[Service]\nExecStart=/usr/local/bin/ciproxy-cache-sidecar machine --listen 0.0.0.0:7887 --admin-listen 127.0.0.1:8229 --cache-dir /var/lib/ciproxy/area/cache --admin-token-file /var/lib/ciproxy/cache-admin-token\nRestart=on-failure\nRestartSec=5\nKillMode=control-group\nTimeoutStopSec=30\n\n[Install]\nWantedBy=multi-user.target\n";

#[async_trait]
pub(crate) trait CacheService: Send + Sync {
    async fn install(&self) -> Result<()>;
}

pub(crate) struct ReleasedCache {
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) release_server: String,
    pub(crate) starter: Arc<dyn CacheStart>,
}

#[async_trait]
impl CacheService for ReleasedCache {
    async fn install(&self) -> Result<()> {
        private_directory(Path::new("/var/lib/ciproxy"))?;
        let token_path = Path::new("/var/lib/ciproxy/cache-admin-token");
        if !token_path.exists() {
            let mut bytes = [0u8; 32];
            getrandom::fill(&mut bytes)
                .context("create machine cache administration credential")?;
            let token = Secret::new(hex::encode(bytes));
            atomic_private_write(token_path, token.expose_secret().as_bytes())?;
        }
        let old_version = std::fs::read_to_string(VERSION).unwrap_or_else(|_| "0.0.0".to_owned());
        let unit_changed = !std::fs::read_to_string("/etc/systemd/system/ciproxy-cache.service")
            .is_ok_and(|current| current == UNIT);
        let restart = old_version.trim() != env!("CARGO_PKG_VERSION")
            || !Path::new(BINARY).exists()
            || unit_changed;
        if restart {
            if !Path::new(BINARY).exists() {
                atomic_private_write(Path::new(BINARY), &[])?;
            }
            let updater = Updater::new(
                Arc::new(
                    SecureReleases::new(self.release_server.clone())
                        .context("configure cache release client")?,
                ),
                Arc::new(CheckedInstaller {
                    installer: Arc::new(TarGzBinaryInstaller::new())
                        as Arc<dyn BinaryInstaller + Send + Sync>,
                }),
            );
            let installed = updater
                .upgrade(UpdateRequest {
                    check: UpdateCheck {
                        binary: "ciproxy-cache-sidecar".to_owned(),
                        current_version: old_version.trim().to_owned(),
                        target: "x86_64-unknown-linux-gnu".to_owned(),
                    },
                    current_exe: BINARY.into(),
                    force: true,
                })
                .await
                .context("install released machine cache")?;
            if let cli_updater::UpgradeOutcome::Installed(asset) = installed {
                atomic_private_write(Path::new(VERSION), asset.latest.version.as_bytes())?;
            }
        }
        std::fs::write("/etc/systemd/system/ciproxy-cache.service", UNIT)
            .context("install mount-bound cache unit")?;

        self.commands
            .run("systemctl", &arguments(&["daemon-reload"]), &[])
            .await?;
        self.starter.start(restart).await?;
        Ok(())
    }
}
