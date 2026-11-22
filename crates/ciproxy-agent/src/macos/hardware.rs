//! The hardware report and the install check of a Mac, from facts that the
//! macOS host reads with `sysctlbyname` and `ciproxy-vz`.

use std::fmt;

use ciproxy_protocol::platform::{Platform, Virtualization};
use ciproxy_protocol::reports::{DiskReport, HardwareReport};

use super::paths::AREA;
use crate::error::Error;

/// A macOS release, such as 26.7.2.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MacosVersion {
    pub(crate) major: u32,
    pub(crate) minor: u32,
    pub(crate) patch: u32,
}

impl MacosVersion {
    /// Parse `kern.osproductversion`, such as `26.7` or `26.7.3`.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().split('.');
        let mut number = |required: bool| match parts.next() {
            Some(part) => part.parse().ok(),
            None => (!required).then_some(0),
        };
        let version = Self {
            major: number(true)?,
            minor: number(false)?,
            patch: number(false)?,
        };
        parts.next().is_none().then_some(version)
    }
}

impl fmt::Display for MacosVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The macOS of the planned macOS image. Virtualization.framework does not
/// run a guest newer than its host, so install refuses an older Mac. The
/// metadata of a newer image can raise this minimum after install.
pub(crate) const MINIMUM: MacosVersion = MacosVersion {
    major: 26,
    minor: 7,
    patch: 2,
};

/// What the Mac reports about itself.
#[derive(Clone, Debug)]
pub(crate) struct MacFacts {
    /// `machdep.cpu.brand_string`, such as `Apple M4`.
    pub(crate) processor: String,
    /// `hw.logicalcpu`.
    pub(crate) threads: u32,
    /// `hw.memsize`.
    pub(crate) memory_bytes: u64,
    /// `kern.osproductversion`.
    pub(crate) version: MacosVersion,
    /// `VZVirtualMachine.isSupported`.
    pub(crate) framework_supported: bool,
    /// Whether the running binary has `com.apple.security.virtualization`.
    pub(crate) entitlement: bool,
}

/// What the newest local image needs from its host.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImageNeeds {
    pub(crate) minimum_host: MacosVersion,
    /// `VZMacHardwareModel.isSupported` for the image's hardware model.
    pub(crate) hardware_model_supported: bool,
}

/// The space of the agent's APFS volume.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AreaSpace {
    /// The volume quota, or the container size when the quota is larger.
    pub(crate) capacity_bytes: u64,
    /// The free bytes of the volume, at most the free bytes of its container.
    pub(crate) free_bytes: u64,
}

pub(crate) fn virtualization(facts: &MacFacts, image: Option<ImageNeeds>) -> Virtualization {
    let minimum = image.map_or(MINIMUM, |image| image.minimum_host.max(MINIMUM));
    if !facts.framework_supported {
        Virtualization::FrameworkUnsupported
    } else if !facts.entitlement {
        Virtualization::EntitlementMissing
    } else if facts.version < minimum {
        Virtualization::OperatingSystemTooOld
    } else if image.is_some_and(|image| !image.hardware_model_supported) {
        Virtualization::FrameworkUnsupported
    } else {
        Virtualization::Available
    }
}

/// The report has one disk: the agent's APFS volume. A Mac with no volume
/// yet reports no disk, so the hosted runtime knows no job disk capacity.
pub(crate) fn report(
    facts: &MacFacts,
    virtualization: Virtualization,
    area: Option<AreaSpace>,
) -> HardwareReport {
    HardwareReport {
        platform: Platform::MacosArm64,
        processor: facts.processor.clone(),
        threads: facts.threads,
        memory_bytes: facts.memory_bytes,
        virtualization,
        disks: area
            .map(|area| DiskReport {
                name: AREA.to_owned(),
                capacity_bytes: area.capacity_bytes,
                free_bytes: area.free_bytes.min(area.capacity_bytes),
            })
            .into_iter()
            .collect(),
    }
}

/// Install runs only as root on Apple Silicon with the minimum macOS.
pub(crate) fn install_check(root: bool, arm64: bool, version: MacosVersion) -> Result<(), Error> {
    if !root || !arm64 {
        return Err(Error::MacHostUnsupported);
    }
    if version < MINIMUM {
        return Err(Error::MacosVersion {
            current: version.to_string(),
            minimum: MINIMUM.to_string(),
        });
    }
    Ok(())
}
