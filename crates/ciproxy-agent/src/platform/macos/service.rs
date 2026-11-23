//! The launchd agent daemon. VM helpers are other jobs with their own
//! lifetimes, so a stop or a restart of the agent keeps running jobs.

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::area;
use crate::boundaries::Service;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::area_access::{Access, FULL_DISK_ACCESS_REPAIR, clear_result, wait_result};
use crate::macos::launchd::{agent_plist, launchctl, newsyslog_conf};
use crate::macos::paths::{
    AGENT_PLIST, BINARY, BINARY_FOLDER, CONTROL, LOG, LOG_FOLDER, NEWSYSLOG,
};
use crate::persistence::private_directory;

/// How long install waits for the daemon's access check.
const ACCESS_WAIT: Duration = Duration::from_secs(30);

pub(crate) struct LaunchdService;

impl Service for LaunchdService {
    fn install(&self) -> Result<()> {
        install_binary()?;
        private_directory(Path::new(LOG_FOLDER))?;
        write_public(Path::new(NEWSYSLOG), newsyslog_conf().as_bytes())?;
        write_public(Path::new(AGENT_PLIST), agent_plist().as_bytes())?;
        let other_disk = area::record()?.is_some_and(|record| record.disk.is_some());
        if other_disk {
            clear_result(Path::new(CONTROL))?;
        }
        if loaded() {
            launchctl_run(&launchctl::restart_agent())?;
        } else {
            launchctl_run(&launchctl::bootstrap_agent())?;
            launchctl_run(&launchctl::enable_agent())?;
        }
        if other_disk {
            daemon_access()?;
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        if loaded() {
            launchctl_run(&launchctl::bootout_agent())?;
        }
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        self.stop()?;
        for path in [
            AGENT_PLIST,
            NEWSYSLOG,
            BINARY,
            "/usr/local/bin/ciproxy-agent.pending",
        ] {
            match fs::remove_file(path) {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    return Err(error).context("remove agent installation");
                }
                _ => {}
            }
        }
        match fs::remove_dir_all(LOG_FOLDER) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                Err(error).context("remove agent logs")
            }
            _ => Ok(()),
        }
    }
}

/// Wait for the daemon's first access check of an area on another disk.
/// The daemon writes the result at start, so it comes within seconds. With
/// no result, install goes on and names the log.
fn daemon_access() -> Result<()> {
    match wait_result(Path::new(CONTROL), ACCESS_WAIT, Duration::from_millis(250)) {
        Some(Access::Open) => Ok(()),
        Some(Access::Blocked) => Err(Error::Preparation {
            component: "area volume",
            repair: FULL_DISK_ACCESS_REPAIR,
        }),
        None => {
            tracing::warn!(
                log = LOG,
                "the agent has not checked its area volume yet; check the log"
            );
            Ok(())
        }
    }
}

/// Copy the running binary to a staging name, then rename it into place.
/// A rename gives a new file. macOS caches a code signature for each file,
/// so a write into an existing signed binary makes the kernel stop it.
///
/// A root daemon runs this binary, so only root may change it or its
/// folders. On macOS, `fs::copy` clones the file and keeps the source
/// owner, so install sets root:wheel and mode 0755 itself.
fn install_binary() -> Result<()> {
    if !Path::new(BINARY_FOLDER).is_dir() {
        fs::create_dir_all(BINARY_FOLDER).context("create /usr/local/bin")?;
        for folder in ["/usr/local", BINARY_FOLDER] {
            fs::set_permissions(folder, fs::Permissions::from_mode(0o755))
                .context("open /usr/local/bin")?;
        }
    }
    for folder in ["/usr/local", BINARY_FOLDER] {
        root_only_folder(Path::new(folder))?;
    }
    let current = std::env::current_exe().context("locate agent binary")?;
    if current == Path::new(BINARY) {
        return root_executable(Path::new(BINARY));
    }
    let pending = format!("{BINARY}.pending");
    fs::copy(&current, &pending).context("stage agent binary")?;
    root_executable(Path::new(&pending))?;
    fs::rename(&pending, BINARY).context("install agent binary")
}

fn root_executable(path: &Path) -> Result<()> {
    std::os::unix::fs::chown(path, Some(0), Some(0)).context("give the agent binary to root")?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).context("protect agent binary")
}

/// A user who owns or can write a folder of the binary could replace the
/// binary by a rename, and the root daemon would run it.
fn root_only_folder(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).context("inspect a binary folder")?;
    if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
        return Err(Error::Preparation {
            component: "/usr/local/bin",
            repair: "Make /usr/local and /usr/local/bin owned by root and not writable by group or others: sudo chown root:wheel /usr/local /usr/local/bin && sudo chmod 755 /usr/local /usr/local/bin. Then rerun install.",
        });
    }
    Ok(())
}

/// launchd reads its files as root and refuses one that others can write.
fn write_public(path: &Path, bytes: &[u8]) -> Result<()> {
    let staged = path.with_extension("pending");
    fs::write(&staged, bytes).context("write a launchd file")?;
    fs::set_permissions(&staged, fs::Permissions::from_mode(0o644))
        .context("protect a launchd file")?;
    fs::rename(&staged, path).context("publish a launchd file")
}

fn loaded() -> bool {
    Command::new("launchctl")
        .args(launchctl::print_agent())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn launchctl_run(args: &[String]) -> Result<()> {
    let status = Command::new("launchctl")
        .args(args)
        .status()
        .context("control the agent job")?;
    if !status.success() {
        return Err(Error::HostCommand {
            program: "launchctl".to_owned(),
        });
    }
    Ok(())
}
