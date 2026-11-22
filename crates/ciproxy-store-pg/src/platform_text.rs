//! Stored text values of the platform and the virtualization state.

use ciproxy_protocol::platform::{Platform, Virtualization};
use ciproxy_store_interface::error::{Error, Result};

/// The stored text of a platform, equal to its wire value.
pub(crate) fn platform_text(platform: Platform) -> &'static str {
    match platform {
        Platform::LinuxX64 => "linux_x64",
        Platform::MacosArm64 => "macos_arm64",
    }
}

/// Read a stored platform, which a check constraint keeps to known values.
pub(crate) fn parse_platform(text: &str) -> Result<Platform> {
    match text {
        "linux_x64" => Ok(Platform::LinuxX64),
        "macos_arm64" => Ok(Platform::MacosArm64),
        _ => Err(Error::InvalidValue),
    }
}

/// The stored text of a virtualization state, equal to its wire value.
pub(crate) fn virtualization_text(virtualization: Virtualization) -> &'static str {
    match virtualization {
        Virtualization::Available => "available",
        Virtualization::KvmUnavailable => "kvm_unavailable",
        Virtualization::FrameworkUnsupported => "framework_unsupported",
        Virtualization::EntitlementMissing => "entitlement_missing",
        Virtualization::OperatingSystemTooOld => "operating_system_too_old",
    }
}

/// Read a stored virtualization state.
pub(crate) fn parse_virtualization(text: &str) -> Result<Virtualization> {
    [
        Virtualization::Available,
        Virtualization::KvmUnavailable,
        Virtualization::FrameworkUnsupported,
        Virtualization::EntitlementMissing,
        Virtualization::OperatingSystemTooOld,
    ]
    .into_iter()
    .find(|value| virtualization_text(*value) == text)
    .ok_or(Error::InvalidValue)
}
