//! Whether the daemon can read and write an area volume on another disk.
//!
//! macOS privacy control (TCC) can block a root launchd job from a volume
//! on an external disk, also when the volume is mounted under `/var/db`.
//! TCC sorts the volume by its disk, not by its mount point. Then each file
//! call on the volume fails with "Operation not permitted" or "Permission
//! denied", until the agent binary has Full Disk Access.
//!
//! The daemon checks its access at each start, and writes the result to
//! `area-access` in the control folder, which is on the startup disk, with
//! the start time of its process. Install reads that result after it loads
//! the daemon, and takes only a result of a process that started after
//! that load. Only the daemon's
//! own launchd job gives the real answer: the terminal of an install can
//! have an access that the daemon does not have.

use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::error::{ErrorResultExt, Result};
use crate::persistence::atomic_private_write;

/// The repair of a command from a terminal. macOS checks the terminal app
/// or `sshd` then, not the agent.
pub(crate) const TERMINAL_ACCESS_REPAIR: &str = "macOS blocks this command from the area volume on an external disk. Give Full Disk Access to the terminal app, or to sshd for an SSH session, and to /usr/local/bin/ciproxy-agent in System Settings > Privacy & Security > Full Disk Access. Then run the command again.";

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
/// to root means that macOS blocks the agent. A failed listing for another
/// cause is an error. A failed write for another cause, such as a full or
/// read-only volume, only gives a warning: the listing already passed, and
/// later steps of the start can free space.
pub(crate) fn probe(area: &Path) -> Result<Access> {
    let listed = std::fs::read_dir(area).and_then(|mut entries| entries.next().transpose());
    match listed {
        Ok(_) => {}
        Err(error) if denied(&error) => return Ok(Access::Blocked),
        Err(error) => return Err(error).context("check access to the area volume"),
    }
    let check = area.join(CHECK);
    match std::fs::write(&check, b"").and_then(|()| std::fs::remove_file(&check)) {
        Ok(()) => Ok(Access::Open),
        Err(error) if denied(&error) => Ok(Access::Blocked),
        Err(error) => {
            tracing::warn!(%error, "cannot write a check file on the area volume");
            Ok(Access::Open)
        }
    }
}

fn denied(error: &std::io::Error) -> bool {
    let denied = error.kind() == std::io::ErrorKind::PermissionDenied;
    if denied {
        tracing::warn!(%error, "the agent cannot use the area volume");
    }
    denied
}

/// The time when this process started its first access check, or an
/// earlier time that [`mark_start`] gave.
fn process_start() -> SystemTime {
    static START: OnceLock<SystemTime> = OnceLock::new();
    *START.get_or_init(SystemTime::now)
}

/// Keep the start time of this process, as early as possible.
pub(crate) fn mark_start() {
    process_start();
}

fn millis(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Write the result with the start time of this process, so install can
/// tell it from a result of a daemon process that it restarted.
pub(crate) fn record_result(control: &Path, access: Access) -> Result<()> {
    let word = match access {
        Access::Open => "open",
        Access::Blocked => "blocked",
    };
    let text = format!("{word} {}\n", millis(process_start()));
    atomic_private_write(&control.join(RESULT), text.as_bytes())
}

/// The result of a process that started at `since` or later.
pub(crate) fn read_result(control: &Path, since: SystemTime) -> Option<Access> {
    let text = std::fs::read_to_string(control.join(RESULT)).ok()?;
    let (word, started) = text.trim().split_once(' ')?;
    if started.parse::<u128>().ok()? < millis(since) {
        return None;
    }
    match word {
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

/// Wait until a daemon that started at `since` or later writes its result,
/// or until `timeout` ends.
pub(crate) fn wait_result(
    control: &Path,
    since: SystemTime,
    timeout: Duration,
    poll: Duration,
) -> Option<Access> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(access) = read_result(control, since) {
            return Some(access);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(poll);
    }
}
