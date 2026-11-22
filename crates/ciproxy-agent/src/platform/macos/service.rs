//! The launchd agent daemon. VM helpers are other jobs with their own
//! lifetimes, so a stop or a restart of the agent keeps running jobs.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::boundaries::Service;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::launchd::{agent_plist, launchctl, newsyslog_conf};
use crate::macos::paths::{AGENT_PLIST, BINARY, BINARY_FOLDER, LOG_FOLDER, NEWSYSLOG};
use crate::persistence::private_directory;

pub(crate) struct LaunchdService;

impl Service for LaunchdService {
    fn install(&self) -> Result<()> {
        install_binary()?;
        private_directory(Path::new(LOG_FOLDER))?;
        write_public(Path::new(NEWSYSLOG), newsyslog_conf().as_bytes())?;
        write_public(Path::new(AGENT_PLIST), agent_plist().as_bytes())?;
        if loaded() {
            launchctl_run(&launchctl::restart_agent())
        } else {
            launchctl_run(&launchctl::bootstrap_agent())?;
            launchctl_run(&launchctl::enable_agent())
        }
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

/// Copy the running binary to a staging name, then rename it into place.
/// A rename gives a new file. macOS caches a code signature for each file,
/// so a write into an existing signed binary makes the kernel stop it.
fn install_binary() -> Result<()> {
    if !Path::new(BINARY_FOLDER).is_dir() {
        fs::create_dir_all(BINARY_FOLDER).context("create /usr/local/bin")?;
        fs::set_permissions(BINARY_FOLDER, fs::Permissions::from_mode(0o755))
            .context("open /usr/local/bin")?;
    }
    let current = std::env::current_exe().context("locate agent binary")?;
    if current == Path::new(BINARY) {
        return Ok(());
    }
    let pending = format!("{BINARY}.pending");
    fs::copy(&current, &pending).context("stage agent binary")?;
    fs::set_permissions(&pending, fs::Permissions::from_mode(0o755))
        .context("open agent binary")?;
    fs::rename(&pending, BINARY).context("install agent binary")
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
