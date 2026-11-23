//! Connect to a vsock port of the guest.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::NSError;
use objc2_virtualization::{VZVirtioSocketConnection, VZVirtioSocketDevice};

use super::vm::Vm;
use super::{Once, POSIX_ERROR_DOMAIN, Sendable, ns_error, receive};
use crate::error::{Error, Result};

impl Vm {
    /// Connect to vsock `port` of the guest and return the stream socket.
    ///
    /// The socket is a duplicate, so it stays open after the framework
    /// releases its connection object, and it can pass to another process.
    /// It has `FD_CLOEXEC` set. When the guest does not listen on the port
    /// yet, this fails with [`Error::GuestNotListening`]. Try again later.
    pub async fn connect(&self, port: u32) -> Result<OwnedFd> {
        let (once, receiver) = Once::new();
        let block = RcBlock::new(
            move |connection: *mut VZVirtioSocketConnection, error: *mut NSError| {
                // SAFETY: The framework passes null or valid objects.
                let result = match unsafe { (connection.as_ref(), error.as_ref()) } {
                    (Some(connection), _) => duplicate(connection),
                    (None, Some(error)) => Err(connect_error(port, error)),
                    (None, None) => Err(Error::NoResult),
                };
                once.send(result);
            },
        );
        let block = Sendable(block);
        let inner = Arc::clone(&self.inner);
        self.inner.queue.exec_async(move || {
            let block = block.get();
            match socket_device(&inner) {
                // SAFETY: Runs on the VM's queue with a completion block.
                Some(device) => unsafe { device.connectToPort_completionHandler(port, block) },
                None => block.call((std::ptr::null_mut(), std::ptr::null_mut())),
            }
        });
        receive(receiver).await
    }
}

fn socket_device(inner: &super::vm::Inner) -> Option<Retained<VZVirtioSocketDevice>> {
    // SAFETY: A property read on the VM's queue.
    let devices = unsafe { inner.machine().socketDevices() };
    devices
        .firstObject()?
        .downcast::<VZVirtioSocketDevice>()
        .ok()
}

fn duplicate(connection: &VZVirtioSocketConnection) -> Result<OwnedFd> {
    // SAFETY: A property read. The connection owns the descriptor.
    let descriptor = unsafe { connection.fileDescriptor() };
    // SAFETY: `fcntl` with `F_DUPFD_CLOEXEC` makes a new descriptor and
    // leaves the old one to the connection.
    let duplicate = unsafe { libc::fcntl(descriptor, libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        return Err(Error::Io {
            action: "duplicate the vsock socket",
            path: "vsock".into(),
            source: std::io::Error::last_os_error(),
        });
    }
    // SAFETY: `duplicate` is a new descriptor that nothing else owns.
    Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
}

/// A refused or reset connect means that the guest does not listen yet.
fn connect_error(port: u32, error: &NSError) -> Error {
    let refused = [libc::ECONNREFUSED, libc::ECONNRESET].map(|code| code as isize);
    if error.domain().to_string() == POSIX_ERROR_DOMAIN && refused.contains(&error.code()) {
        return Error::GuestNotListening { port };
    }
    ns_error(error)
}
