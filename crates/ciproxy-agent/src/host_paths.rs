//! The control directory is outside the one owned workload filesystem.

use std::path::PathBuf;

use crate::host_release::VERSION;

pub(crate) const CONTROL: &str = "/var/lib/ciproxy";
pub(crate) const AREA: &str = "/var/lib/ciproxy/area";
pub(crate) const BACKING: &str = "/var/lib/ciproxy/area.xfs";
pub(crate) const MOUNT: &str = "var-lib-ciproxy-area.mount";
pub(crate) const MOUNT_PATH: &str = "/etc/systemd/system/var-lib-ciproxy-area.mount";

pub(crate) fn tools() -> PathBuf {
    PathBuf::from(CONTROL)
        .join("tools")
        .join(format!("v{VERSION}"))
}

pub(crate) fn mount_unit() -> String {
    format!(
        "# ciproxy owned XFS area v1\n[Unit]\nDescription=ciproxy workload area\nRequiresMountsFor={CONTROL}\nBefore=ciproxy-agent.service\n\n[Mount]\nWhat={BACKING}\nWhere={AREA}\nType=xfs\nOptions=loop,rw,nosuid\nDirectoryMode=0700\n\n[Install]\nWantedBy=multi-user.target\n"
    )
}
