//! Private cache installation files and lazy checked release work.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::secret::Secret;
use cli_updater::{TarGzBinaryInstaller, UpdateRequest, Updater, UpgradeOutcome};

use crate::error::{ErrorResultExt, Result};
use crate::persistence::{atomic_private_write, private_directory};
use crate::release_client::SecureReleases;
use crate::updates::CheckedInstaller;

const BINARY: &str = "/usr/local/bin/ciproxy-cache-sidecar";
const VERSION: &str = "/var/lib/ciproxy/cache-sidecar-version";

/// Non-secret installed facts needed to plan cache activation and repair.
pub struct InstalledCache {
    /// Installed executable path.
    pub(crate) binary: PathBuf,
    /// Last successfully installed release version.
    pub(crate) version: String,
    /// Whether the installed executable exists.
    pub(crate) binary_exists: bool,
}

#[cfg_attr(test, unimock::unimock(api = CacheInstallFilesMock))]
/// Existing private files used by cache installation and release repair.
pub(crate) trait CacheInstallFiles: Send + Sync {
    /// Prepare the private credential and read non-secret installed facts.
    fn prepare(&self) -> Result<InstalledCache>;
    /// Create the missing executable placeholder needed by checked installation.
    fn stage_binary(&self) -> Result<()>;
    /// Record a successfully installed version.
    fn record_version(&self, version: &str) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = CacheReleaseMock))]
#[async_trait]
/// Checked release work that starts only after cache protection.
pub(crate) trait CacheRelease: Send + Sync {
    /// Fetch and install a checked release while preserving its source error.
    async fn upgrade(&self, request: UpdateRequest) -> Result<UpgradeOutcome>;
}

pub(crate) struct LinuxCacheInstallFiles;

pub(crate) struct CheckedCacheRelease {
    pub(crate) release_server: String,
}

impl CacheInstallFiles for LinuxCacheInstallFiles {
    fn prepare(&self) -> Result<InstalledCache> {
        private_directory(Path::new("/var/lib/ciproxy"))?;
        let token_path = Path::new("/var/lib/ciproxy/cache-admin-token");
        if !token_path.exists() {
            let mut bytes = [0u8; 32];
            getrandom::fill(&mut bytes)
                .context("create machine cache administration credential")?;
            let token = Secret::new(hex::encode(bytes));
            atomic_private_write(token_path, token.expose_secret().as_bytes())?;
        }
        Ok(InstalledCache {
            binary: BINARY.into(),
            version: std::fs::read_to_string(VERSION).unwrap_or_else(|_| "0.0.0".to_owned()),
            binary_exists: Path::new(BINARY).exists(),
        })
    }

    fn stage_binary(&self) -> Result<()> {
        atomic_private_write(Path::new(BINARY), &[])
    }

    fn record_version(&self, version: &str) -> Result<()> {
        atomic_private_write(Path::new(VERSION), version.as_bytes())
    }
}

#[async_trait]
impl CacheRelease for CheckedCacheRelease {
    async fn upgrade(&self, request: UpdateRequest) -> Result<UpgradeOutcome> {
        let updater = Updater::new(
            Arc::new(
                SecureReleases::new(self.release_server.clone())
                    .context("configure cache release client")?,
            ),
            Arc::new(CheckedInstaller {
                installer: Arc::new(TarGzBinaryInstaller::new()),
            }),
        );
        updater
            .upgrade(request)
            .await
            .context("install released machine cache")
    }
}
