//! The fixed Ethernet address of a VM's NAT network device.

use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// A 48-bit Ethernet address.
///
/// The text form is six two-digit lower-case hex octets with colons, such as
/// `a2:4c:08:01:02:0f`. That is the form `VZMACAddress` reads and writes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Make an address from its six octets.
    pub const fn new(octets: [u8; 6]) -> Self {
        Self(octets)
    }

    /// Make a random unicast address with the locally administered bit set.
    pub fn random_local() -> Result<Self> {
        let mut octets = [0_u8; 6];
        getrandom::fill(&mut octets).map_err(|error| Error::Random(error.to_string()))?;
        Ok(Self::from_random(octets))
    }

    /// Clear the multicast bit and set the locally administered bit.
    pub(crate) fn from_random(mut octets: [u8; 6]) -> Self {
        octets[0] = (octets[0] & 0b1111_1100) | 0b0000_0010;
        Self(octets)
    }

    /// The six octets.
    pub const fn octets(self) -> [u8; 6] {
        self.0
    }

    /// The text form of `/var/db/dhcpd_leases`, with no leading zero in an
    /// octet, such as `a2:4c:8:1:2:f`.
    pub fn lease_form(self) -> String {
        self.0
            .iter()
            .map(|octet| format!("{octet:x}"))
            .collect::<Vec<_>>()
            .join(":")
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        write!(formatter, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{f:02x}")
    }
}

/// Read six colon-separated hex octets of one or two digits each.
///
/// So it reads both the normal form and the lease form.
impl FromStr for MacAddress {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self> {
        let invalid = || Error::InvalidMacAddress(text.to_owned());
        let mut octets = [0_u8; 6];
        let mut parts = text.trim().split(':');
        for octet in &mut octets {
            let part = parts.next().ok_or_else(invalid)?;
            if part.is_empty() || part.len() > 2 || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid());
            }
            *octet = u8::from_str_radix(part, 16).map_err(|_| invalid())?;
        }
        if parts.next().is_some() {
            return Err(invalid());
        }
        Ok(Self(octets))
    }
}

#[cfg(test)]
#[path = "_tests_/mac_address_tests.rs"]
mod mac_address_tests;
