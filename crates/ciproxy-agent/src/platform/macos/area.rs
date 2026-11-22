//! The agent's own APFS volume, mounted at `/var/db/ciproxy/area`.
//!
//! The volume has a quota, so jobs and images cannot fill the Mac's disk.
//! `area.json` records its UUID. Install and uninstall act only on the
//! volume with that UUID. An `/etc/fstab` line mounts it again at boot.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::paths::{AREA, CONTROL};
use crate::persistence::{atomic_private_write, private_directory};

const NAME: &str = "ciproxy";
const FSTAB: &str = "/etc/fstab";
/// The smallest volume that holds one image, its download, and one job.
const MINIMUM_BYTES: u64 = 150 * 1024 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AreaRecord {
    pub(crate) volume_uuid: String,
    pub(crate) quota_bytes: u64,
}

fn record_path() -> PathBuf {
    Path::new(CONTROL).join("area.json")
}

pub(crate) fn record() -> Result<Option<AreaRecord>> {
    match std::fs::read(record_path()) {
        Ok(bytes) => Ok(Some(
            serde_json::from_slice(&bytes).context("decode the area record")?,
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("read the area record"),
    }
}

/// Create the volume once, or mount the recorded one.
pub(crate) fn prepare(requested_bytes: Option<u64>) -> Result<AreaRecord> {
    if let Some(record) = record()? {
        mount(&record)?;
        return Ok(record);
    }
    let container = info_value("/System/Volumes/Data", "APFSContainerReference")?;
    let free: u64 = info_value("/System/Volumes/Data", "APFSContainerFree")?
        .parse()
        .context("read the container free space")?;
    let quota = requested_bytes.unwrap_or(free / 4 * 3);
    if quota < MINIMUM_BYTES || quota > free {
        return Err(Error::Preparation {
            component: "area volume",
            repair: "Free at least 200 GB on the startup disk, or set CIPROXY_AREA_BYTES to a smaller size of at least 150 GB. Then rerun install.",
        });
    }
    private_directory(Path::new(AREA))?;
    let output = diskutil(&[
        "apfs",
        "addVolume",
        &container,
        "APFS",
        NAME,
        "-quota",
        &format!("{quota}B"),
        "-nomount",
    ])?;
    let device = new_device(&output).ok_or_else(|| area_error("read the new volume"))?;
    let record = AreaRecord {
        volume_uuid: info_value(&device, "VolumeUUID")?,
        quota_bytes: quota,
    };
    atomic_private_write(
        &record_path(),
        &serde_json::to_vec(&record).context("encode the area record")?,
    )?;
    add_fstab_line(&record.volume_uuid)?;
    mount(&record)?;
    for (program, args) in [
        ("mdutil", vec!["-i", "off", AREA]),
        ("tmutil", vec!["addexclusion", "-v", AREA]),
    ] {
        if !run_quiet(program, &args) {
            tracing::warn!(
                program,
                "the area volume keeps its default indexing or backup"
            );
        }
    }
    Ok(record)
}

/// Mount the recorded volume when it is not mounted at the area path.
pub(crate) fn mount(record: &AreaRecord) -> Result<()> {
    if mounted(record) {
        return Ok(());
    }
    private_directory(Path::new(AREA))?;
    diskutil(&["mount", "-mountPoint", AREA, &record.volume_uuid])?;
    if mounted(record) {
        Ok(())
    } else {
        Err(area_error("mount the area volume"))
    }
}

/// The volume at the area path is the recorded one.
pub(crate) fn mounted(record: &AreaRecord) -> bool {
    info_value(AREA, "MountPoint").is_ok_and(|point| point == AREA)
        && info_value(AREA, "VolumeUUID").is_ok_and(|uuid| uuid == record.volume_uuid)
}

/// Delete the recorded volume, its fstab line, and the record.
pub(crate) fn remove() -> Result<()> {
    let Some(record) = record()? else {
        return Ok(());
    };
    if info_value(&record.volume_uuid, "VolumeName").is_ok_and(|name| name == NAME) {
        diskutil(&["apfs", "deleteVolume", &record.volume_uuid])?;
    }
    remove_fstab_line(&record.volume_uuid)?;
    std::fs::remove_file(record_path()).context("remove the area record")?;
    match std::fs::remove_dir(AREA) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove the area mount point")
        }
        _ => Ok(()),
    }
}

fn fstab_line(uuid: &str) -> String {
    format!("UUID={uuid} {AREA} apfs rw,nobrowse")
}

fn add_fstab_line(uuid: &str) -> Result<()> {
    let text = std::fs::read_to_string(FSTAB).unwrap_or_default();
    if text.lines().any(|line| line.trim() == fstab_line(uuid)) {
        return Ok(());
    }
    let mut next = text;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&fstab_line(uuid));
    next.push('\n');
    std::fs::write(FSTAB, next).context("add the area to /etc/fstab")
}

fn remove_fstab_line(uuid: &str) -> Result<()> {
    let Ok(text) = std::fs::read_to_string(FSTAB) else {
        return Ok(());
    };
    let kept: String = text
        .lines()
        .filter(|line| line.trim() != fstab_line(uuid))
        .map(|line| format!("{line}\n"))
        .collect();
    if kept != text {
        std::fs::write(FSTAB, kept).context("remove the area from /etc/fstab")?;
    }
    Ok(())
}

/// The device in `diskutil apfs addVolume` output, such as `disk3s7`.
fn new_device(output: &str) -> Option<String> {
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

/// One value of `diskutil info -plist TARGET`, read with `plutil`.
fn info_value(target: &str, key: &str) -> Result<String> {
    let plist = Command::new("diskutil")
        .args(["info", "-plist", target])
        .output()
        .context("run diskutil info")?;
    if !plist.status.success() {
        return Err(area_error("read volume facts"));
    }
    let mut child = Command::new("plutil")
        .args(["-extract", key, "raw", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("run plutil")?;
    if let Some(mut stdin) = child.stdin.take() {
        std::io::Write::write_all(&mut stdin, &plist.stdout).context("send volume facts")?;
    }
    let output = child.wait_with_output().context("wait for plutil")?;
    if !output.status.success() {
        return Err(area_error("read a volume fact"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn diskutil(args: &[&str]) -> Result<String> {
    let output = Command::new("diskutil")
        .args(args)
        .output()
        .context("run diskutil")?;
    if !output.status.success() {
        tracing::warn!(
            stderr = %String::from_utf8_lossy(&output.stderr).trim(),
            "diskutil failed"
        );
        return Err(area_error("run diskutil"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_quiet(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn area_error(step: &'static str) -> Error {
    tracing::warn!(step, "an area volume step failed");
    Error::HostCommand {
        program: "diskutil".to_owned(),
    }
}
