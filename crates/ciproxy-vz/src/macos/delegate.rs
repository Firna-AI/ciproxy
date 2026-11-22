//! The VM delegate that reports when a VM stops by itself.

use std::sync::Mutex;

use futures_channel::oneshot;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_foundation::{NSError, NSObjectProtocol};
use objc2_virtualization::{VZVirtualMachine, VZVirtualMachineDelegate};

use super::ns_error;
use super::vm::StopReason;

/// The stop state of one run of a VM, and the callers that wait for it.
#[derive(Default)]
pub(crate) struct StopState {
    inner: Mutex<StopInner>,
}

#[derive(Default)]
struct StopInner {
    reason: Option<StopReason>,
    waiters: Vec<oneshot::Sender<StopReason>>,
}

impl StopState {
    /// Forget the last stop, because a new run starts.
    pub(crate) fn reset(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.reason = None;
        }
    }

    /// Record the stop and wake every waiter. Only the first stop of a run
    /// counts.
    pub(crate) fn finish(&self, reason: StopReason) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        if inner.reason.is_some() {
            return;
        }
        inner.reason = Some(reason.clone());
        for waiter in inner.waiters.drain(..) {
            let _receiver_gone = waiter.send(reason.clone());
        }
    }

    /// The reason at once if the run has stopped, or else a receiver.
    pub(crate) fn wait(&self) -> Result<StopReason, oneshot::Receiver<StopReason>> {
        let (sender, receiver) = oneshot::channel();
        match self.inner.lock() {
            Ok(mut inner) => match &inner.reason {
                Some(reason) => Ok(reason.clone()),
                None => {
                    inner.waiters.push(sender);
                    Err(receiver)
                }
            },
            Err(_) => Ok(StopReason::Error("the stop state is poisoned".to_owned())),
        }
    }
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and this class
    // implements no `dealloc`.
    #[unsafe(super(NSObject))]
    #[name = "CiproxyVzStopDelegate"]
    #[ivars = StopState]
    pub(crate) struct StopDelegate;

    unsafe impl NSObjectProtocol for StopDelegate {}

    unsafe impl VZVirtualMachineDelegate for StopDelegate {
        #[unsafe(method(guestDidStopVirtualMachine:))]
        fn guest_did_stop(&self, _machine: &VZVirtualMachine) {
            self.ivars().finish(StopReason::Guest);
        }

        #[unsafe(method(virtualMachine:didStopWithError:))]
        fn did_stop_with_error(&self, _machine: &VZVirtualMachine, error: &NSError) {
            self.ivars()
                .finish(StopReason::Error(ns_error(error).to_string()));
        }
    }
);

impl StopDelegate {
    pub(crate) fn new() -> Retained<Self> {
        let this = Self::alloc().set_ivars(StopState::default());
        // SAFETY: `init` of NSObject with initialized ivars.
        unsafe { msg_send![super(this), init] }
    }
}
