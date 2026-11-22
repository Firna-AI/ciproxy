//! Find the IPv4 address of a VM in the DHCP leases of the macOS NAT.
//!
//! The NAT of Virtualization.framework gets its addresses from `bootpd`.
//! `bootpd` writes each lease to [`LEASE_FILE`], which every user can read.
//! Each lease is one block:
//!
//! ```text
//! {
//!     name=admin-vm
//!     ip_address=192.168.64.3
//!     hw_address=1,a2:4c:8:1:2:f
//!     identifier=1,a2:4c:8:1:2:f
//!     lease=0x6740a1b2
//! }
//! ```
//!
//! `hw_address` starts with the hardware type and a comma. Its octets have
//! no leading zero. `lease` is the hex Unix time when the lease ends.

use std::net::Ipv4Addr;
use std::path::Path;

use crate::error::{Error, Result};
use crate::mac_address::MacAddress;

/// The lease file of `bootpd` on macOS.
pub const LEASE_FILE: &str = "/var/db/dhcpd_leases";

/// One DHCP lease.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lease {
    /// The host name that the guest sent, if any.
    pub name: Option<String>,
    /// The leased address.
    pub ip_address: Ipv4Addr,
    /// The Ethernet address of the guest.
    pub hardware: MacAddress,
    /// The Unix time in seconds when the lease ends, or 0 when not given.
    pub expires: u64,
}

/// Read every complete lease in the text of a lease file.
///
/// A block with no valid `ip_address` or `hw_address` is skipped, because
/// it can name no VM.
pub fn parse(text: &str) -> Vec<Lease> {
    let mut leases = Vec::new();
    let mut block: Option<Fields> = None;
    for line in text.lines().map(str::trim) {
        match line {
            "{" => block = Some(Fields::default()),
            "}" => leases.extend(block.take().and_then(Fields::lease)),
            _ => {
                if let (Some(fields), Some((key, value))) = (block.as_mut(), line.split_once('=')) {
                    fields.set(key.trim(), value.trim());
                }
            }
        }
    }
    leases
}

/// The address of the newest lease for `mac` in the text of a lease file.
pub fn find(text: &str, mac: MacAddress) -> Option<Ipv4Addr> {
    parse(text)
        .into_iter()
        .filter(|lease| lease.hardware == mac)
        .max_by_key(|lease| lease.expires)
        .map(|lease| lease.ip_address)
}

/// Read the lease file at `path` and find the address for `mac`.
///
/// A missing file gives `None`, because `bootpd` creates the file only
/// after its first lease.
pub fn lookup(path: &Path, mac: MacAddress) -> Result<Option<Ipv4Addr>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(find(&text, mac)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::Io {
            action: "read the DHCP lease file",
            path: path.to_owned(),
            source,
        }),
    }
}

#[derive(Default)]
struct Fields {
    name: Option<String>,
    ip_address: Option<Ipv4Addr>,
    hardware: Option<MacAddress>,
    expires: u64,
}

impl Fields {
    fn set(&mut self, key: &str, value: &str) {
        match key {
            "name" => self.name = Some(value.to_owned()),
            "ip_address" => self.ip_address = value.parse().ok(),
            "hw_address" => {
                let address = value.split_once(',').map_or(value, |(_, address)| address);
                self.hardware = address.parse().ok();
            }
            "lease" => {
                let digits = value.trim_start_matches("0x");
                self.expires = u64::from_str_radix(digits, 16).unwrap_or(0);
            }
            _ => {}
        }
    }

    fn lease(self) -> Option<Lease> {
        Some(Lease {
            name: self.name,
            ip_address: self.ip_address?,
            hardware: self.hardware?,
            expires: self.expires,
        })
    }
}

#[cfg(test)]
#[path = "_tests_/lease_tests.rs"]
mod lease_tests;
