//! One macOS VM on its own serial dispatch queue.

use std::mem::ManuallyDrop;
use std::path::Path;
use std::sync::Arc;

use dispatch2::{DispatchQueue, DispatchQueueAttr, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass};
use objc2_virtualization::{
    VZMacOSVirtualMachineStartOptions, VZVirtualMachine, VZVirtualMachineConfiguration,
    VZVirtualMachineState,
};

use super::configuration::{build, check_save_restore};
use super::delegate::StopDelegate;
use super::{Sendable, error_block, file_url, receive};
use crate::bundle::Bundle;
use crate::error::{Error, Result};
use crate::mac_address::MacAddress;
use crate::settings::VmSettings;

/// Why a run of a VM ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StopReason {
    /// The guest turned itself off, for example with `shutdown -h now`.
    Guest,
    /// The host stopped the VM with [`Vm::stop`].
    Host,
    /// The VM stopped because of an error.
    Error(String),
}

/// The state of a VM, as `VZVirtualMachineState` gives it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VmState {
    /// Not running. [`Vm::start`] or [`Vm::restore`] can run it.
    Stopped,
    /// Running.
    Running,
    /// Paused. [`Vm::resume`] or [`Vm::save`] can follow.
    Paused,
    /// Stopped after an unrecoverable error.
    Error,
    /// Between two states.
    Changing,
}

/// One macOS VM. It is `Send` and `Sync`.
///
/// The VM runs inside this process. It stops when the last handle drops or
/// the process exits.
pub struct Vm {
    pub(crate) inner: Arc<Inner>,
}

pub(crate) struct Inner {
    pub(crate) queue: DispatchRetained<DispatchQueue>,
    machine: ManuallyDrop<Sendable<Retained<VZVirtualMachine>>>,
    /// Immutable once the VM exists, so any thread may read it.
    configuration: Sendable<Retained<VZVirtualMachineConfiguration>>,
    /// The ivars use a mutex, and the VM calls it on its queue.
    delegate: Sendable<Retained<StopDelegate>>,
    mac: MacAddress,
}

impl Vm {
    /// Make the VM of `bundle` on a new private serial queue.
    ///
    /// This checks the files and the settings. It does not start the VM.
    pub fn new(bundle: &Bundle, settings: &VmSettings) -> Result<Self> {
        let configuration = build(bundle, settings)?;
        Self::from_configuration(configuration, bundle.read_mac_address()?, bundle)
    }

    pub(crate) fn from_configuration(
        configuration: Retained<VZVirtualMachineConfiguration>,
        mac: MacAddress,
        bundle: &Bundle,
    ) -> Result<Self> {
        let label = format!("build.ciproxy.vz.{}", bundle.root().display());
        let queue = DispatchQueue::new(&label, DispatchQueueAttr::SERIAL);
        let delegate = Sendable(StopDelegate::new());
        let configuration = Sendable(configuration);
        let mut machine = None;
        queue.exec_sync(|| {
            // SAFETY: The VM is made on its own queue, and every later call
            // to it runs on that queue.
            let created = unsafe {
                VZVirtualMachine::initWithConfiguration_queue(
                    VZVirtualMachine::alloc(),
                    configuration.get(),
                    &queue,
                )
            };
            // SAFETY: `Inner` retains the delegate for the life of the VM,
            // and the VM calls it on its queue.
            unsafe { created.setDelegate(Some(ProtocolObject::from_ref(&**delegate.get()))) };
            machine = Some(Sendable(created));
        });
        let machine = machine.ok_or(Error::NoResult)?;
        Ok(Self {
            inner: Arc::new(Inner {
                queue,
                machine: ManuallyDrop::new(machine),
                configuration,
                delegate,
                mac,
            }),
        })
    }

    /// The MAC address of the NAT device. Use it with [`crate::lease`].
    pub fn mac_address(&self) -> MacAddress {
        self.inner.mac
    }

    /// The current state.
    pub fn state(&self) -> VmState {
        let mut state = VmState::Changing;
        self.inner.queue.exec_sync(|| {
            // SAFETY: A property read on the VM's queue.
            state = map_state(unsafe { self.inner.machine().state() });
        });
        state
    }

    /// Start the VM from its disk.
    pub async fn start(&self) -> Result<()> {
        self.inner.delegate.0.ivars().reset();
        // SAFETY: Runs on the VM's queue with a completion block.
        self.call(|machine, block| unsafe { machine.startWithCompletionHandler(block) })
            .await
    }

    /// Start the VM in macOS Recovery, for example to run `csrutil`.
    pub async fn start_recovery(&self) -> Result<()> {
        self.inner.delegate.0.ivars().reset();
        // SAFETY: A setter on a new, unshared object, then a start on the
        // VM's queue with a completion block.
        self.call(|machine, block| unsafe {
            let options = VZMacOSVirtualMachineStartOptions::new();
            options.setStartUpFromMacOSRecovery(true);
            machine.startWithOptions_completionHandler(&options, block)
        })
        .await
    }

    /// Stop the VM at once, as a power cut does.
    pub async fn stop(&self) -> Result<()> {
        // SAFETY: Runs on the VM's queue with a completion block.
        self.call(|machine, block| unsafe { machine.stopWithCompletionHandler(block) })
            .await?;
        self.inner.delegate.0.ivars().finish(StopReason::Host);
        Ok(())
    }

    /// Ask the guest to turn itself off, as the power button does.
    ///
    /// [`Vm::stopped`] reports [`StopReason::Guest`] when it is off.
    pub fn request_stop(&self) -> Result<()> {
        let mut result = Err(Error::NoResult);
        self.inner.queue.exec_sync(|| {
            // SAFETY: Runs on the VM's queue.
            result = unsafe { self.inner.machine().requestStopWithError() }
                .map_err(|error| super::ns_error(&error));
        });
        result
    }

    /// Pause a running VM.
    pub async fn pause(&self) -> Result<()> {
        // SAFETY: Runs on the VM's queue with a completion block.
        self.call(|machine, block| unsafe { machine.pauseWithCompletionHandler(block) })
            .await
    }

    /// Resume a paused VM.
    pub async fn resume(&self) -> Result<()> {
        // SAFETY: Runs on the VM's queue with a completion block.
        self.call(|machine, block| unsafe { machine.resumeWithCompletionHandler(block) })
            .await
    }

    /// Save the state of a paused VM to `path`. The VM stays paused.
    ///
    /// The file works only on this Mac, only with the same bundle and
    /// settings, and for only one VM at a time.
    pub async fn save(&self, path: &Path) -> Result<()> {
        self.check_save_restore()?;
        let url = Sendable(file_url(path)?);
        // SAFETY: Runs on the VM's queue with a completion block.
        self.call(move |machine, block| unsafe {
            machine.saveMachineStateToURL_completionHandler(url.get(), block)
        })
        .await
    }

    /// Restore a stopped VM from a state file. The VM is then paused, so
    /// call [`Vm::resume`] next.
    pub async fn restore(&self, path: &Path) -> Result<()> {
        self.check_save_restore()?;
        self.inner.delegate.0.ivars().reset();
        let url = Sendable(file_url(path)?);
        // SAFETY: Runs on the VM's queue with a completion block.
        self.call(move |machine, block| unsafe {
            machine.restoreMachineStateFromURL_completionHandler(url.get(), block)
        })
        .await
    }

    /// Wait until the current run stops by itself or by [`Vm::stop`].
    ///
    /// Call it after a successful start or restore. It returns at once if
    /// the run already stopped. Many callers can wait.
    pub async fn stopped(&self) -> StopReason {
        match self.inner.delegate.0.ivars().wait() {
            Ok(reason) => reason,
            Err(receiver) => receiver
                .await
                .unwrap_or_else(|_| StopReason::Error("the VM was dropped".to_owned())),
        }
    }

    fn check_save_restore(&self) -> Result<()> {
        check_save_restore(&self.inner.configuration.0)
    }

    /// Run `operation` on the queue and wait for its completion block.
    pub(crate) async fn call<F>(&self, operation: F) -> Result<()>
    where
        F: FnOnce(&VZVirtualMachine, &block2::DynBlock<dyn Fn(*mut objc2_foundation::NSError)>)
            + Send
            + 'static,
    {
        let (block, receiver) = error_block();
        let block = Sendable(block);
        let inner = Arc::clone(&self.inner);
        self.inner.queue.exec_async(move || {
            operation(inner.machine(), block.get());
        });
        receive(receiver).await
    }
}

impl Inner {
    /// The VM. Call it only on `queue`.
    pub(crate) fn machine(&self) -> &VZVirtualMachine {
        &self.machine.0
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        // SAFETY: `machine` is never used again after this take.
        let machine = unsafe { ManuallyDrop::take(&mut self.machine) };
        // Release the VM on its own queue.
        self.queue.exec_async(move || drop(machine));
    }
}

fn map_state(state: VZVirtualMachineState) -> VmState {
    match state {
        VZVirtualMachineState::Stopped => VmState::Stopped,
        VZVirtualMachineState::Running => VmState::Running,
        VZVirtualMachineState::Paused => VmState::Paused,
        VZVirtualMachineState::Error => VmState::Error,
        _ => VmState::Changing,
    }
}

/// Fail the build when the handle or its futures stop being `Send`.
#[allow(dead_code)]
fn assert_send(vm: &Vm, path: &Path) {
    fn send<T: Send>(_: T) {}
    send(vm.start());
    send(vm.start_recovery());
    send(vm.stop());
    send(vm.save(path));
    send(vm.restore(path));
    send(vm.stopped());
    send(vm.connect(5252));
    send(super::RestoreImage::latest());
    send(super::RestoreImage::load(path));
}
