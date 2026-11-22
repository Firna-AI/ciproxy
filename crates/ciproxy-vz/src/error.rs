//! The errors of this crate.

use std::path::PathBuf;

/// The result type of this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error from the VM layer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The text is not a MAC address.
    #[error("invalid MAC address {0:?}")]
    InvalidMacAddress(String),
    /// The operating system gave no random bytes.
    #[error("no random bytes: {0}")]
    Random(String),
    /// A file operation failed.
    #[error("could not {action} at {}: {source}", path.display())]
    Io {
        /// What the operation tried to do.
        action: &'static str,
        /// The file or directory.
        path: PathBuf,
        /// The cause.
        #[source]
        source: std::io::Error,
    },
    /// A bundle file holds a value that is not valid.
    #[error("invalid bundle file {}: {reason}", path.display())]
    InvalidBundle {
        /// The file.
        path: PathBuf,
        /// What is wrong.
        reason: String,
    },
    /// The VM settings do not meet the needs of the guest or the host.
    #[error("invalid VM settings: {0}")]
    InvalidSettings(String),
    /// This computer cannot run macOS VMs, or the API needs a newer macOS.
    #[error("not supported on this computer: {0}")]
    Unsupported(String),
    /// macOS already runs the most macOS VMs that its licence allows.
    ///
    /// The limit is 2 active macOS VMs on one Mac. A paused VM still counts,
    /// so stop another VM first. After this error, the VM is in the error
    /// state. This is `VZErrorVirtualMachineLimitExceeded`.
    #[error("the Mac already runs the most macOS VMs that macOS allows: {message}")]
    VirtualMachineLimit {
        /// The text from Virtualization.framework.
        message: String,
    },
    /// The guest does not listen on the vsock port yet. Try again later.
    #[error("the guest does not listen on vsock port {port}")]
    GuestNotListening {
        /// The guest port.
        port: u32,
    },
    /// Another error from Virtualization.framework or Foundation.
    #[error("{domain} error {code}: {message}")]
    Framework {
        /// The `NSError` domain, such as `VZErrorDomain`.
        domain: String,
        /// The `NSError` code.
        code: isize,
        /// The localized description.
        message: String,
    },
    /// The framework dropped an operation without a result.
    #[error("the operation ended without a result")]
    NoResult,
}
