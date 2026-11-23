//! `diskutil` calls and their plist output, read with `plutil`.

use std::process::{Command, Stdio};

use crate::error::{Error, ErrorResultExt, Result};

/// The plist output of one `diskutil … -plist` call.
pub(super) struct Plist(Vec<u8>);

impl Plist {
    /// `diskutil info -plist TARGET`. A target can be a mount point, a
    /// device, or a volume UUID.
    pub(super) fn info(target: &str) -> Result<Self> {
        Self::run(&["info", "-plist", target])
    }

    /// `diskutil apfs list -plist CONTAINER`.
    pub(super) fn container(container: &str) -> Result<Self> {
        Self::run(&["apfs", "list", "-plist", container])
    }

    fn run(args: &[&str]) -> Result<Self> {
        let output = Command::new("diskutil")
            .args(args)
            .output()
            .context("run diskutil")?;
        if !output.status.success() {
            tracing::warn!(
                args = ?args,
                output = %String::from_utf8_lossy(&output.stdout).trim(),
                stderr = %String::from_utf8_lossy(&output.stderr).trim(),
                "diskutil failed"
            );
            return Err(area_error("read volume facts"));
        }
        Ok(Self(output.stdout))
    }

    /// One value, such as `MountPoint` or `Containers.0.CapacityFree`.
    pub(super) fn value(&self, key: &str) -> Result<String> {
        self.optional(key)?
            .ok_or_else(|| area_error("read a volume fact"))
    }

    /// One value, or none when the plist does not have the key.
    pub(super) fn optional(&self, key: &str) -> Result<Option<String>> {
        let mut child = Command::new("plutil")
            .args(["-extract", key, "raw", "-o", "-", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("run plutil")?;
        if let Some(mut stdin) = child.stdin.take() {
            std::io::Write::write_all(&mut stdin, &self.0).context("send volume facts")?;
        }
        let output = child.wait_with_output().context("wait for plutil")?;
        Ok(output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned()))
    }

    /// A boolean value. A missing key gives none.
    pub(super) fn flag(&self, key: &str) -> Result<Option<bool>> {
        Ok(self.optional(key)?.map(|value| value == "true"))
    }

    pub(super) fn number(&self, key: &str) -> Result<u64> {
        self.value(key)?
            .parse()
            .map_err(|_| area_error("read a volume size"))
    }
}

/// One value of `diskutil info -plist TARGET`.
pub(super) fn info_value(target: &str, key: &str) -> Result<String> {
    Plist::info(target)?.value(key)
}

pub(super) fn diskutil(args: &[&str]) -> Result<String> {
    let output = Command::new("diskutil")
        .args(args)
        .output()
        .context("run diskutil")?;
    if !output.status.success() {
        tracing::warn!(
            args = ?args,
            output = %String::from_utf8_lossy(&output.stdout).trim(),
            stderr = %String::from_utf8_lossy(&output.stderr).trim(),
            "diskutil failed"
        );
        return Err(area_error("run diskutil"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The device in `diskutil apfs addVolume` output, such as `disk3s7`.
pub(super) fn new_device(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .rev()
        .find(|word| {
            word.strip_prefix("disk").is_some_and(|rest| {
                rest.contains('s') && rest.bytes().all(|b| b.is_ascii_alphanumeric())
            })
        })
        .map(str::to_owned)
}

/// Run a tool. Return its output when it fails, for the log.
pub(super) fn run_checked(program: &str, args: &[&str]) -> Option<String> {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => None,
        Ok(output) => Some(format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => Some(error.to_string()),
    }
}

pub(super) fn area_error(step: &'static str) -> Error {
    tracing::warn!(step, "an area volume step failed");
    Error::HostCommand {
        program: "diskutil".to_owned(),
    }
}
