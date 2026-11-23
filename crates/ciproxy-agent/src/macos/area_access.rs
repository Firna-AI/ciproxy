//! Whether the daemon can read and write an area volume on another disk.
//!
//! macOS privacy control (TCC) can block a root launchd job from a volume
//! on an external disk, also when the volume is mounted under `/var/db`.
//! TCC sorts the volume by its disk, not by its mount point. Then each file
//! call on the volume fails with "Operation not permitted" or "Permission
//! denied", until the agent binary has Full Disk Access.
//!
//! The daemon checks its access at each start, and writes the result to
//! `area-access` in the control folder, which is on the startup disk.
//! Install reads that result after it loads the daemon. Only the daemon's
//! own launchd job gives the real answer: the terminal of an install can
//! have an access that the daemon does not have.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::error::{ErrorResultExt, Result};
use crate::persistence::atomic_private_write;

pub(crate) const FULL_DISK_ACCESS_REPAIR: &str = "macOS blocks the agent from the area volume on an external disk. Give /usr/local/bin/ciproxy-agent Full Disk Access in System Settings > Privacy & Security > Full Disk Access. The agent then starts by itself within 5 seconds.";

/// The result of one access check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Access {
    Open,
    Blocked,
}

/// The result file in the control folder.
const RESULT: &str = "area-access";
/// A file that the check writes and removes at the volume root.
const CHECK: &str = ".ciproxy-access-check";

/// List the volume root, then write and remove one file there. A refusal
/// to root means that macOS blocks the agent. Another failure is an error.
pub(crate) fn probe(area: &Path) -> Result<Access> {
    match probe_files(area) {
        Ok(()) => Ok(Access::Open),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            tracing::warn!(%error, "the agent cannot use the area volume");
            Ok(Access::Blocked)
        }
        Err(error) => Err(error).context("check access to the area volume"),
    }
}

fn probe_files(area: &Path) -> std::io::Result<()> {
    std::fs::read_dir(area)?.next().transpose()?;
    let check = area.join(CHECK);
    std::fs::write(&check, b"")?;
    std::fs::remove_file(&check)
}

pub(crate) fn record_result(control: &Path, access: Access) -> Result<()> {
    let text: &[u8] = match access {
        Access::Open => b"open\n",
        Access::Blocked => b"blocked\n",
    };
    atomic_private_write(&control.join(RESULT), text)
}

pub(crate) fn read_result(control: &Path) -> Option<Access> {
    match std::fs::read_to_string(control.join(RESULT)).ok()?.trim() {
        "open" => Some(Access::Open),
        "blocked" => Some(Access::Blocked),
        _ => None,
    }
}

pub(crate) fn clear_result(control: &Path) -> Result<()> {
    match std::fs::remove_file(control.join(RESULT)) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove the area access result")
        }
        _ => Ok(()),
    }
}

/// Wait until the daemon writes its result, or until `timeout` ends.
pub(crate) fn wait_result(control: &Path, timeout: Duration, poll: Duration) -> Option<Access> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(access) = read_result(control) {
            return Some(access);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(poll);
    }
}
