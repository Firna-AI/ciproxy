//! Paths and commands that differ between the Linux and macOS job images.

use std::ffi::{CStr, CString};
use std::path::Path;

use tokio::process::Command;

use crate::error::{Error, ErrorResultExt, Result};

/// The operating system of the job image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Platform {
    /// A Firecracker VM or a Boat sandbox with Docker.
    Linux,
    /// A Virtualization.framework VM on an own Mac, with no Docker.
    MacOs,
}

/// Environment values that the runner entry receives. Nothing else passes.
pub(crate) const RUNNER_ENVIRONMENT: [&str; 2] = [
    "ACTIONS_RUNNER_INPUT_JITCONFIG",
    "CIPROXY_ASSIGNMENT_SOCKET",
];

impl Platform {
    /// The platform of this build.
    pub(crate) const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }

    /// Private directory of the source-hook socket. macOS has no `/run`.
    pub(crate) const fn assignment_directory(self) -> &'static str {
        match self {
            Self::Linux => "/run/ciproxy",
            Self::MacOs => "/var/run/ciproxy",
        }
    }

    /// Path of the source-hook socket in the assignment directory.
    pub(crate) const fn assignment_socket(self) -> &'static str {
        match self {
            Self::Linux => "/run/ciproxy/assignment.sock",
            Self::MacOs => "/var/run/ciproxy/assignment.sock",
        }
    }

    /// Whether the image has Docker, its bridge, and the Docker disk.
    pub(crate) const fn docker(self) -> bool {
        matches!(self, Self::Linux)
    }

    /// Listen address of the VM cache proxy. Linux containers reach it on the
    /// Docker bridge. macOS steps run in the VM itself, so loopback suffices.
    pub(crate) const fn proxy_listen(self) -> &'static str {
        match self {
            Self::Linux => "0.0.0.0:8230",
            Self::MacOs => "127.0.0.1:8230",
        }
    }

    /// Command that ends the VM after the runner ends. x86 Firecracker ends
    /// its process on a reset. A Virtualization.framework VM stops when the
    /// guest powers off, and a reboot would start the guest again.
    pub(crate) const fn end_command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Linux => ("/sbin/reboot", &[]),
            Self::MacOs => ("/sbin/shutdown", &["-h", "now"]),
        }
    }

    /// Command that starts the runner entry as the runner account.
    ///
    /// Linux uses `runuser`. macOS has no `runuser`. There, `launchctl asuser`
    /// puts the runner in the account's launchd domain, so per-user services
    /// such as the simulator and the keychain are reachable. Then `sudo`
    /// changes the account and keeps only the named runner values.
    pub(crate) fn runner_command(self, account: &Account, entry: &Path) -> Command {
        let mut command;
        match self {
            Self::Linux => {
                command = Command::new("/usr/sbin/runuser");
                command.args(["--user", &account.name, "--"]);
            }
            Self::MacOs => {
                command = Command::new("/bin/launchctl");
                command
                    .args(["asuser", &account.uid.to_string(), "/usr/bin/sudo"])
                    .arg("--non-interactive")
                    .arg(format!("--user={}", account.name))
                    .arg("--set-home")
                    .arg(format!("--preserve-env={}", RUNNER_ENVIRONMENT.join(",")))
                    .arg("--");
            }
        }
        command.arg(entry).env_clear();
        command
    }
}

/// The local account that runs the stock runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Account {
    pub(crate) name: String,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
}

impl Account {
    /// Read the account from the user database of the image.
    pub(crate) fn lookup(name: &str) -> Result<Self> {
        let user = CString::new(name).context("runner account name")?;
        let mut entry = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut buffer = vec![0_u8; 16_384];
        let mut found = std::ptr::null_mut();
        // SAFETY: Every pointer refers to live, writable memory of the given
        // size, and `found` is read only after the call returns.
        let status = unsafe {
            libc::getpwnam_r(
                user.as_ptr(),
                entry.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut found,
            )
        };
        if status != 0 || found.is_null() {
            return Err(Error::internal_message("runner account is missing"));
        }
        // SAFETY: A successful lookup with a non-null result fills `entry`.
        let entry = unsafe { entry.assume_init() };
        // SAFETY: `pw_name` points into `buffer`, which is still alive.
        let name = unsafe { CStr::from_ptr(entry.pw_name) }
            .to_string_lossy()
            .into_owned();
        Ok(Self {
            name,
            uid: entry.pw_uid,
            gid: entry.pw_gid,
        })
    }
}

#[cfg(test)]
#[path = "_tests_/platform_tests.rs"]
mod platform_tests;
