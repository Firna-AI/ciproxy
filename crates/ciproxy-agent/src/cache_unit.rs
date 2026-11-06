//! Install the cache listener unit through its own host-file boundary.

use std::path::Path;
use std::sync::Arc;

use crate::cache_network::RULES;
use crate::error::{ErrorResultExt, Result};
use crate::persistence::atomic_private_write;

pub(crate) const UNIT_PATH: &str = "/etc/systemd/system/ciproxy-cache.service";
pub(crate) const POLICY_PATH: &str = "/etc/ciproxy-cache.nft";
pub(crate) const UNIT: &str = "[Unit]\nDescription=ciproxy machine cache\nRequires=var-lib-ciproxy-area.mount\nAfter=var-lib-ciproxy-area.mount network-online.target\nRequiresMountsFor=/var/lib/ciproxy/area/cache\n\n[Service]\nExecStartPre=/usr/sbin/nft -f {policy_path}\nExecStart=/usr/local/bin/ciproxy-cache-sidecar machine --listen 0.0.0.0:7887 --admin-listen 127.0.0.1:8229 --cache-dir /var/lib/ciproxy/area/cache --admin-token-file /var/lib/ciproxy/cache-admin-token\nRestart=on-failure\nRestartSec=5\nKillMode=control-group\nTimeoutStopSec=30\n\n[Install]\nWantedBy=multi-user.target\n";

pub(crate) trait CacheUnit: Send + Sync {
    fn changed(&self) -> bool;
    fn install(&self) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = CacheUnitFilesMock))]
pub(crate) trait CacheUnitFiles: Send + Sync {
    fn read(&self, path: &str) -> Result<Option<String>>;
    fn write(&self, path: &str, contents: &[u8]) -> Result<()>;
}

pub(crate) struct InstalledCacheUnit {
    pub(crate) files: Arc<dyn CacheUnitFiles>,
}

pub(crate) struct LinuxCacheUnitFiles;

impl CacheUnit for InstalledCacheUnit {
    fn changed(&self) -> bool {
        !self
            .files
            .read(UNIT_PATH)
            .is_ok_and(|current| current.as_deref() == Some(definition().as_str()))
    }

    fn install(&self) -> Result<()> {
        self.files.write(POLICY_PATH, RULES.as_bytes())?;
        self.files.write(UNIT_PATH, definition().as_bytes())
    }
}

impl CacheUnitFiles for LinuxCacheUnitFiles {
    fn read(&self, path: &str) -> Result<Option<String>> {
        match std::fs::read_to_string(path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).context("read installed cache unit"),
        }
    }

    fn write(&self, path: &str, contents: &[u8]) -> Result<()> {
        atomic_private_write(Path::new(path), contents)
    }
}

#[cfg(test)]
#[path = "_tests_/cache_unit_tests.rs"]
mod cache_unit_tests;

fn definition() -> String {
    UNIT.replace("{policy_path}", POLICY_PATH)
}
