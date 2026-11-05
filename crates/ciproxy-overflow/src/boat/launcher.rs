//! Fixed privileged image launch with a root-owned once-only marker.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use sandbox_boat::runtime::api::BoatRuntimeApi;
use sandbox_interface::ProviderRef;
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::provider::{GuestLauncher, registration_root};

pub(super) struct ImageLauncher {
    pub(super) runtime: Arc<dyn BoatRuntimeApi>,
}

const PREPARE: &str = r#"
import os, pwd, stat, sys
owner = pwd.getpwnam("sandbox-workload")
root = sys.argv[1]
os.makedirs(root, mode=0o700, exist_ok=True)
os.chown(root, owner.pw_uid, owner.pw_gid)
os.chmod(root, 0o700)
fd = os.open(root + "/registration.json", os.O_WRONLY | os.O_CREAT | os.O_NOFOLLOW, 0o600)
if not stat.S_ISREG(os.fstat(fd).st_mode):
    raise ValueError()
os.fchown(fd, owner.pw_uid, owner.pw_gid)
os.fchmod(fd, 0o600)
os.close(fd)
"#;

#[async_trait]
impl GuestLauncher for ImageLauncher {
    async fn prepare(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<()> {
        let directory = registration_root(delivery_id);
        let command = format!("sudo -n /usr/bin/python3 -I -S -c '{PREPARE}' '{directory}'");
        let result = self
            .runtime
            .helper(sandbox.as_str(), command, Duration::from_secs(30))
            .await
            .internal()?;
        if !result.normal || result.exit_code != Some(0) {
            return Err(Error::ProviderUnavailable);
        }
        Ok(())
    }

    async fn launch(&self, sandbox: ProviderRef, delivery_id: Uuid) -> Result<()> {
        let directory = registration_root(delivery_id);
        let marker = format!("/run/ciproxy-overflow/{}", delivery_id.simple());
        let command = format!(
            "sudo -n /bin/sh -c 'set -eu; install -d -m 0700 /run/ciproxy-overflow; if mkdir {marker} 2>/dev/null; then nohup setsid /bin/bash /opt/ciproxy/rootfs/opt/ciproxy-image/boat-run.sh {directory}/registration.json </dev/null >/dev/null 2>&1 & fi'"
        );
        let result = self
            .runtime
            .helper(sandbox.as_str(), command, Duration::from_secs(30))
            .await
            .internal()?;
        if !result.normal || result.exit_code != Some(0) {
            return Err(Error::ProviderUnavailable);
        }
        Ok(())
    }
}
