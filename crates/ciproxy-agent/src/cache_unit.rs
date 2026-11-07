//! Install the cache listener unit through its own host-file boundary.

use std::path::Path;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::cache_network::RULES;
use crate::error::{ErrorResultExt, Result};
use crate::persistence::atomic_private_write;

pub(crate) const UNIT_PATH: &str = "/etc/systemd/system/ciproxy-cache.service";
pub(crate) const POLICY_PATH: &str = "/etc/ciproxy-cache.nft";
/// Root-owned loader shared by repair and protected systemd starts.
pub(crate) const FIREWALL_PATH: &str = "/etc/ciproxy-cache-firewall";
/// Transient root-owned credential that only root and the cache group can read.
pub(crate) const RUNTIME_TOKEN_PATH: &str = "/run/ciproxy-cache-auth/admin-token";
const DATA_PATH: &str = "/var/lib/ciproxy/area/cache/data";
const FIREWALL: &str = include_str!("../scripts/cache-firewall.sh");
const FINGERPRINT: &str = "CIPROXY_CACHE_UNIT_SHA256";
pub(crate) const UNIT: &str = r#"[Unit]
Description=ciproxy machine cache
Requires=var-lib-ciproxy-area.mount
After=var-lib-ciproxy-area.mount network-online.target
RequiresMountsFor=/var/lib/ciproxy/area/cache

[Service]
User=ciproxy-cache
Group=ciproxy-cache
RuntimeDirectory=ciproxy-cache
RuntimeDirectoryMode=0700
BindPaths={data_path}:/run/ciproxy-cache/data
NoNewPrivileges=yes
CapabilityBoundingSet=
ExecStartPre=+/bin/sh {firewall_path} start
ExecStart=/usr/local/bin/ciproxy-cache-sidecar machine --listen 0.0.0.0:7887 --admin-listen 127.0.0.1:8229 --cache-dir /run/ciproxy-cache/data --admin-token-file {runtime_token_path}
ExecStopPost=+/bin/rm -f {runtime_token_path}
Restart=on-failure
RestartSec=5
KillMode=control-group
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
"#;

pub(crate) trait CacheUnit: Send + Sync {
    fn changed(&self) -> bool;
    fn install(&self) -> Result<()>;
    /// Check the actual main process against the current unit fingerprint.
    fn running_matches(&self, pid: u32) -> bool;
}

#[cfg_attr(test, unimock::unimock(api = CacheUnitFilesMock))]
pub(crate) trait CacheUnitFiles: Send + Sync {
    fn read(&self, path: &str) -> Result<Option<String>>;
    fn write(&self, path: &str, contents: &[u8]) -> Result<()>;
    /// Keep process environment bytes local and return false for unknown proof.
    fn process_matches(&self, pid: u32, fingerprint: &str) -> bool;
}

pub(crate) struct InstalledCacheUnit {
    pub(crate) files: Arc<dyn CacheUnitFiles>,
}

pub(crate) struct LinuxCacheUnitFiles;

impl CacheUnit for InstalledCacheUnit {
    fn running_matches(&self, pid: u32) -> bool {
        self.files.process_matches(pid, &fingerprint())
    }

    fn changed(&self) -> bool {
        !self
            .files
            .read(UNIT_PATH)
            .is_ok_and(|current| current.as_deref() == Some(definition().as_str()))
    }

    fn install(&self) -> Result<()> {
        self.files.write(POLICY_PATH, RULES.as_bytes())?;
        self.files.write(FIREWALL_PATH, FIREWALL.as_bytes())?;
        self.files.write(UNIT_PATH, definition().as_bytes())
    }
}

impl CacheUnitFiles for LinuxCacheUnitFiles {
    fn process_matches(&self, pid: u32, fingerprint: &str) -> bool {
        let Ok(environment) = std::fs::read(format!("/proc/{pid}/environ")) else {
            return false;
        };
        if !environment.ends_with(&[0]) {
            return false;
        }
        let key = format!("{FINGERPRINT}=");
        let expected = format!("{key}{fingerprint}");
        let mut values = environment
            .split(|byte| *byte == 0)
            .filter(|value| value.starts_with(key.as_bytes()));
        values.next() == Some(expected.as_bytes()) && values.next().is_none()
    }

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
    body().replace(
        "[Service]\n",
        &format!("[Service]\nEnvironment={FINGERPRINT}={}\n", fingerprint()),
    )
}

fn fingerprint() -> String {
    hex::encode(Sha256::digest(body().as_bytes()))
}

fn body() -> String {
    UNIT.replace("{firewall_path}", FIREWALL_PATH)
        .replace("{data_path}", DATA_PATH)
        .replace("{runtime_token_path}", RUNTIME_TOKEN_PATH)
}
