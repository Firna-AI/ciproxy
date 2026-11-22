//! `vm.json`, the VM helper's input, and `exit.json`, its result.
//!
//! Both files are root-only and hold no secret. The runner registration
//! reaches the guest only over the vsock stream that the agent holds.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::seats::{SEATS, StartKind};
use super::vm_control::StopReason;

pub(crate) const VM_SPEC_FORMAT: u32 = 1;

/// The VM that one helper creates.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VmSpec {
    pub(crate) format: u32,
    pub(crate) seat: u16,
    pub(crate) cpus: u32,
    pub(crate) memory_bytes: u64,
    /// The VM's own bundle clone in its VM folder.
    pub(crate) bundle: PathBuf,
    pub(crate) start: StartKind,
    /// The state file for [`StartKind::Restore`].
    pub(crate) saved_state: Option<PathBuf>,
}

/// Why the helper refuses a spec before it creates a VM.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum SpecError {
    #[error("the VM file has an unknown format")]
    Format,
    #[error("the VM file names an unknown seat")]
    Seat,
    #[error("the VM file has no CPU or no memory")]
    Size,
    #[error("the VM file must name a state file only for a restore")]
    SavedState,
}

impl VmSpec {
    pub(crate) fn check(&self) -> Result<(), SpecError> {
        if self.format != VM_SPEC_FORMAT {
            return Err(SpecError::Format);
        }
        if self.seat >= SEATS {
            return Err(SpecError::Seat);
        }
        if self.cpus == 0 || self.memory_bytes == 0 {
            return Err(SpecError::Size);
        }
        if (self.start == StartKind::Restore) != self.saved_state.is_some() {
            return Err(SpecError::SavedState);
        }
        Ok(())
    }
}

/// The helper writes this file when its VM stops, before it exits. The
/// agent reads it after a restart that missed the stop.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VmExit {
    pub(crate) reason: ExitReason,
}

/// The stop reason in `exit.json`, in the words of the control socket.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExitReason {
    GuestShutdown,
    HostStop,
    Failure,
}

impl From<StopReason> for ExitReason {
    fn from(reason: StopReason) -> Self {
        match reason {
            StopReason::GuestShutdown => Self::GuestShutdown,
            StopReason::HostStop => Self::HostStop,
            StopReason::Failure => Self::Failure,
        }
    }
}
