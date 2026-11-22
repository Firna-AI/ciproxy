//! macOS VMs on Apple Silicon over Apple's Virtualization.framework.
//!
//! The pure parts build on every platform: [`MacAddress`], the DHCP
//! [`lease`] lookup, the [`Bundle`] files, and the [`VmSettings`] checks.
//! The VM parts build only on macOS. See the crate README for the API and
//! the threading model.

pub mod bundle;
pub mod error;
pub mod lease;
pub mod mac_address;
pub mod settings;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(all(target_os = "macos", feature = "private-vnc"))]
pub use macos::VncServer;
#[cfg(target_os = "macos")]
pub use macos::{
    Requirements, RestoreImage, StopReason, Vm, VmState, clone_bundle, host_limits, install,
};

pub use bundle::{Bundle, Identity};
pub use error::{Error, Result};
pub use mac_address::MacAddress;
pub use settings::{Display, Limits, VmSettings};
