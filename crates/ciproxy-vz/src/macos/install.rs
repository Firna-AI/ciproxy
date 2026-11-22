//! Install macOS from a restore image into a new bundle.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use objc2::AnyThread;
use objc2_virtualization::{
    VZMacAuxiliaryStorage, VZMacAuxiliaryStorageInitializationOptions, VZMacOSInstaller,
};

use super::configuration::{hardware_model, host_limits};
use super::restore_image::RestoreImage;
use super::vm::Vm;
use super::{Sendable, error_block, file_url, ns_error, receive};
use crate::bundle::{Bundle, Identity};
use crate::error::{Error, Result};
use crate::settings::VmSettings;

const PROGRESS_INTERVAL: Duration = Duration::from_secs(1);

/// Create the bundle at `bundle` and install macOS into it from a loaded
/// restore image.
///
/// The bundle directory must not exist yet. `disk_bytes` is the size of the
/// sparse startup disk. `progress` gets the done fraction, from 0.0 to 1.0,
/// about once a second, on a thread of its own. After the install, the VM
/// is stopped, and the Setup Assistant runs on its first start.
pub async fn install(
    bundle: &Bundle,
    image: &RestoreImage,
    settings: &VmSettings,
    disk_bytes: u64,
    progress: impl Fn(f64) + Send + 'static,
) -> Result<()> {
    let requirements = image
        .requirements()
        .ok_or_else(|| Error::Unsupported("this Mac cannot run the restore image".to_owned()))?;
    settings.check(host_limits().and(requirements.limits()))?;
    bundle.create_directory()?;
    bundle.create_disk(disk_bytes)?;
    bundle.write_hardware_model(&requirements.hardware_model)?;
    create_auxiliary_storage(bundle, &requirements.hardware_model)?;
    bundle.write_identity(&Identity::fresh()?)?;
    let vm = Vm::new(bundle, settings)?;
    run_installer(&vm, image, progress).await
}

fn create_auxiliary_storage(bundle: &Bundle, model: &[u8]) -> Result<()> {
    let model = hardware_model(model)?;
    let url = file_url(&bundle.auxiliary_storage())?;
    // SAFETY: A new file URL and a supported hardware model.
    unsafe {
        VZMacAuxiliaryStorage::initCreatingStorageAtURL_hardwareModel_options_error(
            VZMacAuxiliaryStorage::alloc(),
            &url,
            &model,
            VZMacAuxiliaryStorageInitializationOptions::empty(),
        )
    }
    .map(drop)
    .map_err(|error| ns_error(&error))
}

async fn run_installer(
    vm: &Vm,
    image: &RestoreImage,
    progress: impl Fn(f64) + Send + 'static,
) -> Result<()> {
    let url = Sendable(image.file_url());
    let (block, receiver) = error_block();
    let block = Sendable(block);
    let mut started = None;
    vm.inner.queue.exec_sync(|| {
        // SAFETY: The installer is made and started on the VM's queue, as
        // the framework needs, with a local restore image file.
        unsafe {
            let installer = VZMacOSInstaller::initWithVirtualMachine_restoreImageURL(
                VZMacOSInstaller::alloc(),
                vm.inner.machine(),
                url.get(),
            );
            installer.installWithCompletionHandler(block.get());
            started = Some((installer.progress(), Sendable(installer)));
        }
    });
    drop(block);
    let (fraction, installer) = started.ok_or(Error::NoResult)?;
    let done = Arc::new(AtomicBool::new(false));
    let watcher = {
        let done = Arc::clone(&done);
        std::thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                progress(fraction.fractionCompleted());
                std::thread::sleep(PROGRESS_INTERVAL);
            }
            progress(fraction.fractionCompleted());
        })
    };
    let result = receive(receiver).await;
    done.store(true, Ordering::Relaxed);
    let _progress_thread_panicked = watcher.join();
    drop(installer);
    result
}
