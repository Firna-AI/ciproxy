//! The macOS parts over Virtualization.framework.
//!
//! No part needs AppKit or the main thread. Each [`Vm`] owns a private
//! serial dispatch queue, and every call to its `VZVirtualMachine` runs on
//! that queue.

mod clone;
mod configuration;
mod delegate;
mod install;
mod restore_image;
mod vm;
#[cfg(feature = "private-vnc")]
mod vnc;
mod vsock;

use std::path::Path;
use std::sync::Mutex;

use block2::RcBlock;
use futures_channel::oneshot;
use objc2::rc::Retained;
use objc2_foundation::{NSError, NSString, NSURL};
use objc2_virtualization::VZErrorCode;

use crate::error::{Error, Result};

pub use clone::clone_bundle;
pub use configuration::host_limits;
pub use install::install;
pub use restore_image::{Requirements, RestoreImage};
pub use vm::{StopReason, Vm, VmState};
#[cfg(feature = "private-vnc")]
pub use vnc::VncServer;

const VZ_ERROR_DOMAIN: &str = "VZErrorDomain";
const POSIX_ERROR_DOMAIN: &str = "NSPOSIXErrorDomain";

/// Convert an `NSError` into an [`Error`].
pub(crate) fn ns_error(error: &NSError) -> Error {
    let domain = error.domain().to_string();
    let code = error.code();
    let message = error.localizedDescription().to_string();
    if domain == VZ_ERROR_DOMAIN && code == VZErrorCode::VirtualMachineLimitExceeded.0 {
        return Error::VirtualMachineLimit { message };
    }
    Error::Framework {
        domain,
        code,
        message,
    }
}

/// Convert a nullable `NSError` pointer from a completion handler.
///
/// # Safety
///
/// `error` must be null or point to a valid `NSError`.
pub(crate) unsafe fn completion_result(error: *mut NSError) -> Result<()> {
    // SAFETY: The caller passes null or a valid `NSError`.
    match unsafe { error.as_ref() } {
        None => Ok(()),
        Some(error) => Err(ns_error(error)),
    }
}

/// A one-shot result that a completion handler sends once.
pub(crate) struct Once<T> {
    sender: Mutex<Option<oneshot::Sender<T>>>,
}

impl<T> Once<T> {
    pub(crate) fn new() -> (Self, oneshot::Receiver<T>) {
        let (sender, receiver) = oneshot::channel();
        let once = Self {
            sender: Mutex::new(Some(sender)),
        };
        (once, receiver)
    }

    /// Send the value. A second call does nothing.
    pub(crate) fn send(&self, value: T) {
        let sender = self.sender.lock().map(|mut sender| sender.take());
        if let Ok(Some(sender)) = sender {
            let _receiver_gone = sender.send(value);
        }
    }
}

/// Wait for the result of a completion handler.
pub(crate) async fn receive<T>(receiver: oneshot::Receiver<Result<T>>) -> Result<T> {
    receiver.await.map_err(|_| Error::NoResult)?
}

/// A completion block of the `(NSError *)` form.
pub(crate) type ErrorBlock = RcBlock<dyn Fn(*mut NSError)>;

/// A completion block for `(NSError *)` handlers and its receiver.
pub(crate) fn error_block() -> (ErrorBlock, oneshot::Receiver<Result<()>>) {
    let (once, receiver) = Once::new();
    let block = RcBlock::new(move |error: *mut NSError| {
        // SAFETY: Virtualization.framework passes null or a valid `NSError`.
        once.send(unsafe { completion_result(error) });
    });
    (block, receiver)
}

/// A file URL for `path`.
pub(crate) fn file_url(path: &Path) -> Result<Retained<NSURL>> {
    let text = path.to_str().ok_or_else(|| {
        Error::InvalidSettings(format!("the path {} is not UTF-8", path.display()))
    })?;
    Ok(NSURL::fileURLWithPath(&NSString::from_str(text)))
}

/// A value that crosses threads although its type is not `Send`.
///
/// Use it only for Objective-C objects that the code touches on one
/// dispatch queue, or that Apple documents as thread-safe.
pub(crate) struct Sendable<T>(pub(crate) T);

impl<T> Sendable<T> {
    /// The value. A closure that calls this captures the whole wrapper,
    /// not only the field, so the closure stays `Send`.
    pub(crate) fn get(&self) -> &T {
        &self.0
    }
}

// SAFETY: Each user upholds the rule of the type documentation.
unsafe impl<T> Send for Sendable<T> {}
// SAFETY: Each user upholds the rule of the type documentation.
unsafe impl<T> Sync for Sendable<T> {}
