//! The platform of a machine, a label, an image, and a job VM.

use serde::{Deserialize, Serialize};

/// The most macOS VMs that Apple's licence and framework allow on one Mac.
pub const MACOS_VM_LIMIT: u64 = 2;

/// An operating system and processor architecture that ciproxy supports.
///
/// A label, a machine, an image, and a job VM each have one platform. A job
/// VM runs only on a machine, and from an image, of the label's platform.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Linux on x64, with Firecracker VMs over KVM.
    LinuxX64,
    /// macOS on Apple Silicon, with macOS VMs over Virtualization.framework.
    MacosArm64,
}

impl Platform {
    /// The most job VMs that may run at once on one machine of this platform.
    ///
    /// No value means that only slots and job disk limit the VMs.
    pub fn vm_limit(self) -> Option<u64> {
        match self {
            Self::LinuxX64 => None,
            Self::MacosArm64 => Some(MACOS_VM_LIMIT),
        }
    }
}

/// Whether a machine can run job VMs, and the cause when it cannot.
///
/// A machine whose virtualization is not available has no slots.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Virtualization {
    /// The machine can start job VMs.
    Available,
    /// Linux: the agent cannot open `/dev/kvm`.
    KvmUnavailable,
    /// macOS: Virtualization.framework reports no VM support on this Mac.
    FrameworkUnsupported,
    /// macOS: the agent binary lacks the virtualization entitlement.
    EntitlementMissing,
    /// macOS: the Mac's macOS release cannot run the macOS image.
    OperatingSystemTooOld,
}

impl Virtualization {
    /// Whether the machine can start job VMs.
    pub fn is_available(self) -> bool {
        self == Self::Available
    }
}

#[cfg(test)]
#[path = "_tests_/platform_tests.rs"]
mod platform_tests;
