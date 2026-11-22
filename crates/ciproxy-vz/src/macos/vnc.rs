//! A VNC server for the display of a VM, over the private `_VZVNCServer`.
//!
//! Apple does not document `_VZVNCServer`. It can change or go away in any
//! macOS release, and an App Store app must not use it. It listens on every
//! network interface of the Mac, so a caller must block its port with a pf
//! rule or use it only on a trusted network.

use objc2::msg_send;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSString;

use super::Sendable;
use super::vm::Vm;
use crate::error::{Error, Result};

/// A running VNC server. Dropping it stops the server.
pub struct VncServer {
    server: Sendable<Retained<AnyObject>>,
    queue: dispatch2::DispatchRetained<dispatch2::DispatchQueue>,
}

impl Vm {
    /// Serve the display of this VM over VNC on `port`, or on a free port
    /// when `port` is 0.
    ///
    /// With `None`, the server asks for no password. The VM needs a display
    /// in its settings. See the module documentation for the risks of this
    /// private API.
    pub fn start_vnc(&self, port: u16, password: Option<&str>) -> Result<VncServer> {
        let security = security_configuration(password)?;
        let server_class = class("_VZVNCServer")?;
        let queue = self.inner.queue.clone();
        let mut server = None;
        let security = Sendable(security);
        self.inner.queue.exec_sync(|| {
            // SAFETY: The selectors match `_VZVNCServer` on macOS 13 to 26.
            // The server and the VM are used on the VM's queue.
            unsafe {
                let allocated: Allocated<AnyObject> = msg_send![server_class, alloc];
                let created: Option<Retained<AnyObject>> = msg_send![
                    allocated,
                    initWithPort: port,
                    queue: &*self.inner.queue,
                    securityConfiguration: &**security.get(),
                ];
                if let Some(created) = created {
                    let _: () = msg_send![&*created, setVirtualMachine: self.inner.machine()];
                    let _: () = msg_send![&*created, start];
                    server = Some(Sendable(created));
                }
            }
        });
        let server =
            server.ok_or_else(|| Error::Unsupported("_VZVNCServer did not start".to_owned()))?;
        Ok(VncServer { server, queue })
    }
}

impl VncServer {
    /// The TCP port. It is 0 until the server listens.
    pub fn port(&self) -> u16 {
        let mut port = 0;
        self.queue.exec_sync(|| {
            // SAFETY: `port` is a property of `_VZVNCServer`.
            port = unsafe { msg_send![&**self.server.get(), port] };
        });
        port
    }
}

impl Drop for VncServer {
    fn drop(&mut self) {
        self.queue.exec_sync(|| {
            // SAFETY: `stop` is a method of `_VZVNCServer`.
            let _: () = unsafe { msg_send![&**self.server.get(), stop] };
        });
    }
}

fn security_configuration(password: Option<&str>) -> Result<Retained<AnyObject>> {
    // SAFETY: The selectors match the private security configuration
    // classes of `_VZVNCServer`.
    let created: Option<Retained<AnyObject>> = unsafe {
        match password {
            Some(password) => {
                let class = class("_VZVNCAuthenticationSecurityConfiguration")?;
                let allocated: Allocated<AnyObject> = msg_send![class, alloc];
                msg_send![allocated, initWithPassword: &*NSString::from_str(password)]
            }
            None => {
                let class = class("_VZVNCNoSecuritySecurityConfiguration")?;
                msg_send![class, new]
            }
        }
    };
    created.ok_or_else(|| Error::Unsupported("no VNC security configuration".to_owned()))
}

fn class(name: &str) -> Result<&'static AnyClass> {
    let name = std::ffi::CString::new(name).map_err(|_| Error::NoResult)?;
    AnyClass::get(&name).ok_or_else(|| {
        Error::Unsupported(format!(
            "this macOS has no private class {}",
            name.to_string_lossy()
        ))
    })
}
