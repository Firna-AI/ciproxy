//! Fixed macOS paths and launchd labels. See the Paths section of `macos-host.md`.

use std::path::PathBuf;

/// The agent binary. Install creates `/usr/local/bin` when it is missing.
pub(crate) const BINARY: &str = "/usr/local/bin/ciproxy-agent";
pub(crate) const BINARY_FOLDER: &str = "/usr/local/bin";
/// The control directory, the same value as `host_paths::CONTROL` on macOS.
pub(crate) const CONTROL: &str = "/var/db/ciproxy";
/// The mount point of the agent's own APFS volume.
pub(crate) const AREA: &str = "/var/db/ciproxy/area";
/// Root-only settings. launchd has no environment file, so the agent job
/// loads this file through `/bin/sh`.
pub(crate) const ENVIRONMENT: &str = "/etc/ciproxy-agent/environment";
/// The log folder, mode `0700`. The logs hold no secret, but only root reads them.
pub(crate) const LOG_FOLDER: &str = "/var/log/ciproxy";
/// launchd writes the JSON logs of the agent and the VM helpers here.
pub(crate) const LOG: &str = "/var/log/ciproxy/agent.log";
pub(crate) const NEWSYSLOG: &str = "/etc/newsyslog.d/ciproxy.conf";

pub(crate) const AGENT_LABEL: &str = "dev.ciproxy.agent";
pub(crate) const AGENT_PLIST: &str = "/Library/LaunchDaemons/dev.ciproxy.agent.plist";
/// The prefix of every VM helper label. Recovery owns only these labels.
pub(crate) const VM_LABEL_PREFIX: &str = "dev.ciproxy.vm.";

/// The pf anchor. The main pf rules of macOS load every `com.apple/*` anchor.
pub(crate) const PF_ANCHOR: &str = "com.apple/ciproxy";

/// A VM's local ID is 32 lower-case hex digits from a v4 UUID. Paths and
/// labels use it only after this check.
pub(crate) fn valid_local_id(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// The launchd label of the VM helper of one VM.
pub(crate) fn vm_label(local_id: &str) -> String {
    format!("{VM_LABEL_PREFIX}{local_id}")
}

/// The local ID in a label that recovery found, when the label is ours.
pub(crate) fn local_id_of_label(label: &str) -> Option<&str> {
    label
        .strip_prefix(VM_LABEL_PREFIX)
        .filter(|id| valid_local_id(id))
}

/// The helper's launchd file. It is not in `/Library/LaunchDaemons`, so a
/// reboot does not start the helper again.
pub(crate) fn vm_plist(local_id: &str) -> PathBuf {
    PathBuf::from(CONTROL)
        .join("vms")
        .join(format!("{local_id}.plist"))
}

/// The saved state of one VM seat, and the folder that builds the next one.
pub(crate) fn seat_folder(seat: u16, build: bool) -> PathBuf {
    PathBuf::from(AREA)
        .join("seats")
        .join(seat.to_string())
        .join(if build { "build" } else { "current" })
}
