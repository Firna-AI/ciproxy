//! The size and the devices of a VM.

use crate::error::{Error, Result};

/// The size of a VM and its optional display.
///
/// Every VM also gets one NAT virtio network device with the bundle's MAC
/// address, one virtio block startup disk, one entropy device, one memory
/// balloon, one virtio socket device, a keyboard, and a trackpad.
///
/// A saved state restores only into a VM with the same settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmSettings {
    /// The number of virtual CPUs.
    pub cpus: u32,
    /// The memory size in bytes. Virtualization.framework needs a multiple
    /// of 1 MiB.
    pub memory_bytes: u64,
    /// The graphics display, or `None` for a VM with no display.
    pub display: Option<Display>,
}

/// The one graphics display of a VM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Display {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// The pixel density.
    pub pixels_per_inch: u32,
}

impl Default for Display {
    /// 1920 by 1080 at 80 pixels per inch.
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            pixels_per_inch: 80,
        }
    }
}

/// The CPU and memory limits from the host and the guest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// The smallest CPU count.
    pub min_cpus: u32,
    /// The largest CPU count.
    pub max_cpus: u32,
    /// The smallest memory size in bytes.
    pub min_memory_bytes: u64,
    /// The largest memory size in bytes.
    pub max_memory_bytes: u64,
}

const MIB: u64 = 1024 * 1024;

impl VmSettings {
    /// Fail when the settings are outside `limits`, or the memory is not a
    /// whole number of MiB.
    pub fn check(&self, limits: Limits) -> Result<()> {
        if self.cpus < limits.min_cpus || self.cpus > limits.max_cpus {
            return Err(Error::InvalidSettings(format!(
                "{} CPUs is outside {} to {}",
                self.cpus, limits.min_cpus, limits.max_cpus
            )));
        }
        if self.memory_bytes < limits.min_memory_bytes
            || self.memory_bytes > limits.max_memory_bytes
        {
            return Err(Error::InvalidSettings(format!(
                "{} bytes of memory is outside {} to {}",
                self.memory_bytes, limits.min_memory_bytes, limits.max_memory_bytes
            )));
        }
        if !self.memory_bytes.is_multiple_of(MIB) {
            return Err(Error::InvalidSettings(format!(
                "{} bytes of memory is not a whole number of MiB",
                self.memory_bytes
            )));
        }
        if let Some(display) = self.display
            && (display.width == 0 || display.height == 0 || display.pixels_per_inch == 0)
        {
            return Err(Error::InvalidSettings(format!(
                "display {display:?} has a zero value"
            )));
        }
        Ok(())
    }
}

impl Limits {
    /// The narrower of two sets of limits, such as the host and the guest.
    pub fn and(self, other: Self) -> Self {
        Self {
            min_cpus: self.min_cpus.max(other.min_cpus),
            max_cpus: self.max_cpus.min(other.max_cpus),
            min_memory_bytes: self.min_memory_bytes.max(other.min_memory_bytes),
            max_memory_bytes: self.max_memory_bytes.min(other.max_memory_bytes),
        }
    }
}

#[cfg(test)]
#[path = "_tests_/settings_tests.rs"]
mod settings_tests;
