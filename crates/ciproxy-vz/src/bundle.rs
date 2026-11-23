//! The directory that holds the files of one macOS VM.
//!
//! A bundle holds these files:
//!
//! | File | Content |
//! | --- | --- |
//! | `disk.img` | The raw, sparse startup disk |
//! | `aux.img` | The auxiliary storage of the Mac platform |
//! | `hardware-model.bin` | `VZMacHardwareModel.dataRepresentation` |
//! | `machine-identifier.bin` | `VZMacMachineIdentifier.dataRepresentation` |
//! | `mac-address.txt` | The MAC address of the NAT device, one line |
//!
//! The disk and the auxiliary storage belong together. A clone copies both.
//! A released image has only the disk, the auxiliary storage, and the
//! hardware model. Each clone gets its own identity files.

use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::mac_address::MacAddress;

const DISK: &str = "disk.img";
const AUXILIARY_STORAGE: &str = "aux.img";
const HARDWARE_MODEL: &str = "hardware-model.bin";
const MACHINE_IDENTIFIER: &str = "machine-identifier.bin";
const MAC_ADDRESS: &str = "mac-address.txt";

/// The identity of a VM: its MAC address and its machine identifier.
///
/// A saved state restores only into a VM with the same identity. Two VMs
/// that run at the same time need two identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identity {
    /// The MAC address of the NAT device.
    pub mac_address: MacAddress,
    /// `VZMacMachineIdentifier.dataRepresentation`.
    pub machine_identifier: Vec<u8>,
}

/// The paths and the plain files of one VM bundle directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bundle {
    root: PathBuf,
}

impl Bundle {
    /// Name the bundle at `root`. This reads and writes no file.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The bundle directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The raw startup disk.
    pub fn disk(&self) -> PathBuf {
        self.root.join(DISK)
    }

    /// The auxiliary storage of the Mac platform.
    pub fn auxiliary_storage(&self) -> PathBuf {
        self.root.join(AUXILIARY_STORAGE)
    }

    /// The data of the hardware model.
    pub fn hardware_model(&self) -> PathBuf {
        self.root.join(HARDWARE_MODEL)
    }

    /// The data of the machine identifier.
    pub fn machine_identifier(&self) -> PathBuf {
        self.root.join(MACHINE_IDENTIFIER)
    }

    /// The MAC address file.
    pub fn mac_address(&self) -> PathBuf {
        self.root.join(MAC_ADDRESS)
    }

    /// Create the bundle directory. It must not exist yet.
    pub fn create_directory(&self) -> Result<()> {
        std::fs::create_dir(&self.root)
            .map_err(|source| io("create the bundle", &self.root, source))
    }

    /// Create the sparse startup disk of `bytes` bytes. It must not exist yet.
    pub fn create_disk(&self, bytes: u64) -> Result<()> {
        let path = self.disk();
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|source| io("create the disk", &path, source))?;
        file.set_len(bytes)
            .map_err(|source| io("set the disk size", &path, source))
    }

    /// Read the hardware model data.
    pub fn read_hardware_model(&self) -> Result<Vec<u8>> {
        read_data(&self.hardware_model())
    }

    /// Write the hardware model data.
    pub fn write_hardware_model(&self, data: &[u8]) -> Result<()> {
        write_file(&self.hardware_model(), data)
    }

    /// Read the machine identifier data.
    pub fn read_machine_identifier(&self) -> Result<Vec<u8>> {
        read_data(&self.machine_identifier())
    }

    /// Write the machine identifier data.
    pub fn write_machine_identifier(&self, data: &[u8]) -> Result<()> {
        write_file(&self.machine_identifier(), data)
    }

    /// Read the MAC address.
    pub fn read_mac_address(&self) -> Result<MacAddress> {
        let path = self.mac_address();
        let text = std::fs::read_to_string(&path)
            .map_err(|source| io("read the MAC address", &path, source))?;
        text.trim().parse().map_err(|_| Error::InvalidBundle {
            path,
            reason: format!("{:?} is not a MAC address", text.trim()),
        })
    }

    /// Write the MAC address.
    pub fn write_mac_address(&self, mac: MacAddress) -> Result<()> {
        write_file(&self.mac_address(), format!("{mac}\n").as_bytes())
    }

    /// Read the MAC address and the machine identifier.
    pub fn read_identity(&self) -> Result<Identity> {
        Ok(Identity {
            mac_address: self.read_mac_address()?,
            machine_identifier: self.read_machine_identifier()?,
        })
    }

    /// Write the MAC address and the machine identifier.
    pub fn write_identity(&self, identity: &Identity) -> Result<()> {
        self.write_mac_address(identity.mac_address)?;
        self.write_machine_identifier(&identity.machine_identifier)
    }

    /// Fail with the first missing file of a complete bundle.
    pub fn check_files(&self) -> Result<()> {
        self.check_image_files()?;
        check_present([self.machine_identifier(), self.mac_address()])
    }

    /// Fail with the first missing file that a clone takes from its source:
    /// the disk, the auxiliary storage, or the hardware model. A clone gets
    /// its own identity, so its source needs no identity files. A released
    /// image has none.
    pub fn check_image_files(&self) -> Result<()> {
        check_present([self.disk(), self.auxiliary_storage(), self.hardware_model()])
    }
}

fn check_present<const N: usize>(paths: [PathBuf; N]) -> Result<()> {
    for path in paths {
        if !path.is_file() {
            return Err(io(
                "find a bundle file",
                &path,
                std::io::Error::from(ErrorKind::NotFound),
            ));
        }
    }
    Ok(())
}

fn read_data(path: &Path) -> Result<Vec<u8>> {
    let data = std::fs::read(path).map_err(|source| io("read", path, source))?;
    if data.is_empty() {
        return Err(Error::InvalidBundle {
            path: path.to_owned(),
            reason: "the file is empty".to_owned(),
        });
    }
    Ok(data)
}

/// Write through a temporary file and a rename, so a reader never sees half
/// a file.
fn write_file(path: &Path, data: &[u8]) -> Result<()> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    std::fs::write(&temporary, data).map_err(|source| io("write", &temporary, source))?;
    std::fs::rename(&temporary, path).map_err(|source| io("replace", path, source))
}

pub(crate) fn io(action: &'static str, path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        action,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
#[path = "_tests_/bundle_tests.rs"]
mod bundle_tests;
