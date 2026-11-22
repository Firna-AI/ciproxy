//! VM seats and their saved states. See "VM seats and saved states" in `macos-host.md`.
//!
//! A seat is one of the 2 macOS VMs that Apple's licence allows at a time.
//! Each seat keeps a fixed MAC address and machine identifier, so the vmnet
//! DHCP server gives it the same lease, and its saved state can restore.
//! Fresh MAC addresses for each job would use up the address pool, because
//! a lease lasts one day.

use std::time::Duration;

use ciproxy_protocol::platform::MACOS_VM_LIMIT;
use serde::{Deserialize, Serialize};

/// The seats of a Mac, 0 and 1. A seat is the `network_index` of its VM.
pub(crate) const SEATS: u16 = MACOS_VM_LIMIT as u16;

/// A saved state older than this boots cold, and the agent builds it again.
/// The limit keeps the clock step small and stays below the lease time.
pub(crate) const MAX_AGE: Duration = Duration::from_secs(12 * 60 * 60);

/// The lowest seat that no VM and no saved-state build holds.
pub(crate) fn free_seat(held: &[u16]) -> Option<u16> {
    (0..SEATS).find(|seat| !held.contains(seat))
}

/// `saved.json` in `seats/N/current/`. The folder rename publishes it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedState {
    pub(crate) format: u32,
    pub(crate) seat: u16,
    /// The image of the saved VM's disk.
    pub(crate) image: String,
    /// The MAC address of the seat when the agent saved the VM.
    pub(crate) mac_address: String,
    /// The agent that saved the VM. Another version may build another VM
    /// configuration, which cannot restore the state.
    pub(crate) agent_version: String,
    pub(crate) saved_at_unix_seconds: u64,
}

pub(crate) const SAVED_STATE_FORMAT: u32 = 1;

/// What the next job VM of a seat must match to restore a saved state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SeatNeeds<'a> {
    pub(crate) seat: u16,
    pub(crate) newest_image: &'a str,
    pub(crate) mac_address: &'a str,
    pub(crate) agent_version: &'a str,
    pub(crate) now_unix_seconds: u64,
}

/// How a job VM starts.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StartKind {
    /// Boot a clone of the image.
    Cold,
    /// Restore a clone of the seat's saved state.
    Restore,
}

impl SavedState {
    /// A saved state restores only for its own seat, image, identity, and
    /// agent version, and only while it is younger than [`MAX_AGE`]. A save
    /// time in the future also fails, because the clock moved back.
    pub(crate) fn usable(&self, needs: SeatNeeds<'_>) -> bool {
        let age = needs
            .now_unix_seconds
            .checked_sub(self.saved_at_unix_seconds);
        self.format == SAVED_STATE_FORMAT
            && self.seat == needs.seat
            && self.image == needs.newest_image
            && self.mac_address == needs.mac_address
            && self.agent_version == needs.agent_version
            && age.is_some_and(|age| age < MAX_AGE.as_secs())
    }
}

/// Restore when the seat has a usable saved state. Else boot cold.
pub(crate) fn start_kind(saved: Option<&SavedState>, needs: SeatNeeds<'_>) -> StartKind {
    match saved {
        Some(saved) if saved.usable(needs) => StartKind::Restore,
        _ => StartKind::Cold,
    }
}

/// A seat builds a new saved state only when it is free, the machine takes
/// new runners, and the current state cannot restore.
pub(crate) fn needs_build(
    saved: Option<&SavedState>,
    needs: SeatNeeds<'_>,
    seat_free: bool,
    accepting: bool,
) -> bool {
    seat_free && accepting && start_kind(saved, needs) == StartKind::Cold
}
