//! The control directory is outside the one owned workload filesystem.
//!
//! Every platform uses the control directory. The XFS area, its systemd mount,
//! and the Firecracker tools exist only on Linux.

#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use crate::host_release::VERSION;

#[cfg(not(target_os = "macos"))]
pub(crate) const CONTROL: &str = "/var/lib/ciproxy";
/// macOS keeps daemon data in `/var/db`. See `macos-host.md`.
#[cfg(target_os = "macos")]
pub(crate) const CONTROL: &str = "/var/db/ciproxy";
#[cfg(target_os = "linux")]
pub(crate) const AREA: &str = "/var/lib/ciproxy/area";
#[cfg(target_os = "linux")]
pub(crate) const BACKING: &str = "/var/lib/ciproxy/area.xfs";
#[cfg(target_os = "linux")]
pub(crate) const MOUNT: &str = "var-lib-ciproxy-area.mount";
#[cfg(target_os = "linux")]
pub(crate) const MOUNT_PATH: &str = "/etc/systemd/system/var-lib-ciproxy-area.mount";

#[cfg(target_os = "linux")]
pub(crate) fn tools() -> PathBuf {
    PathBuf::from(CONTROL)
        .join("tools")
        .join(format!("v{VERSION}"))
}

#[cfg(target_os = "linux")]
pub(crate) fn mount_unit() -> String {
    format!(
        "# ciproxy owned XFS area v1\n[Unit]\nDescription=ciproxy workload area\nRequiresMountsFor={CONTROL}\nBefore=ciproxy-agent.service\n\n[Mount]\nWhat={BACKING}\nWhere={AREA}\nType=xfs\nOptions=loop,rw,nosuid\nDirectoryMode=0700\n\n[Install]\nWantedBy=multi-user.target\n"
    )
}
