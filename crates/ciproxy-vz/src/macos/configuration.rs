//! Build the `VZVirtualMachineConfiguration` of a bundle.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSData, NSString};
use objc2_virtualization::{
    VZDiskImageStorageDeviceAttachment, VZMACAddress, VZMacAuxiliaryStorage,
    VZMacGraphicsDeviceConfiguration, VZMacGraphicsDisplayConfiguration, VZMacHardwareModel,
    VZMacKeyboardConfiguration, VZMacMachineIdentifier, VZMacOSBootLoader,
    VZMacPlatformConfiguration, VZMacTrackpadConfiguration, VZNATNetworkDeviceAttachment,
    VZVirtioBlockDeviceConfiguration, VZVirtioEntropyDeviceConfiguration,
    VZVirtioNetworkDeviceConfiguration, VZVirtioSocketDeviceConfiguration,
    VZVirtioTraditionalMemoryBalloonDeviceConfiguration, VZVirtualMachineConfiguration,
};

use super::{file_url, ns_error};
use crate::bundle::Bundle;
use crate::error::{Error, Result};
use crate::mac_address::MacAddress;
use crate::settings::{Display, Limits, VmSettings};

/// The CPU and memory limits of Virtualization.framework on this Mac.
pub fn host_limits() -> Limits {
    // SAFETY: These class methods have no preconditions.
    unsafe {
        Limits {
            min_cpus: clamp(VZVirtualMachineConfiguration::minimumAllowedCPUCount()),
            max_cpus: clamp(VZVirtualMachineConfiguration::maximumAllowedCPUCount()),
            min_memory_bytes: VZVirtualMachineConfiguration::minimumAllowedMemorySize(),
            max_memory_bytes: VZVirtualMachineConfiguration::maximumAllowedMemorySize(),
        }
    }
}

fn clamp(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Read the hardware model data of a bundle.
pub(crate) fn hardware_model(data: &[u8]) -> Result<Retained<VZMacHardwareModel>> {
    let data = NSData::with_bytes(data);
    // SAFETY: The initializer accepts any data and returns nil when invalid.
    let model = unsafe {
        VZMacHardwareModel::initWithDataRepresentation(VZMacHardwareModel::alloc(), &data)
    }
    .ok_or_else(|| Error::Unsupported("the hardware model data is not valid".to_owned()))?;
    // SAFETY: A plain property read.
    if !unsafe { model.isSupported() } {
        return Err(Error::Unsupported(
            "this Mac does not support the hardware model".to_owned(),
        ));
    }
    Ok(model)
}

/// Read machine identifier data.
pub(crate) fn machine_identifier(data: &[u8]) -> Result<Retained<VZMacMachineIdentifier>> {
    let data = NSData::with_bytes(data);
    // SAFETY: The initializer accepts any data and returns nil when invalid.
    unsafe {
        VZMacMachineIdentifier::initWithDataRepresentation(VZMacMachineIdentifier::alloc(), &data)
    }
    .ok_or_else(|| Error::Unsupported("the machine identifier data is not valid".to_owned()))
}

/// The data of a new, unique machine identifier.
pub(crate) fn new_machine_identifier() -> Vec<u8> {
    // SAFETY: `new` has no preconditions, and the data is a plain read.
    unsafe { VZMacMachineIdentifier::new().dataRepresentation() }.to_vec()
}

/// Build and validate the configuration of the bundle's VM.
pub(crate) fn build(
    bundle: &Bundle,
    settings: &VmSettings,
) -> Result<Retained<VZVirtualMachineConfiguration>> {
    settings.check(host_limits())?;
    bundle.check_files()?;
    let model = hardware_model(&bundle.read_hardware_model()?)?;
    let identifier = machine_identifier(&bundle.read_machine_identifier()?)?;
    let mac = bundle.read_mac_address()?;
    // SAFETY: Each call below sets a property of a new, unshared object with
    // values of the documented types. Nothing runs the VM yet.
    unsafe {
        let platform = VZMacPlatformConfiguration::new();
        platform.setHardwareModel(&model);
        platform.setMachineIdentifier(&identifier);
        let auxiliary = VZMacAuxiliaryStorage::initWithURL(
            VZMacAuxiliaryStorage::alloc(),
            &*file_url(&bundle.auxiliary_storage())?,
        );
        platform.setAuxiliaryStorage(Some(&auxiliary));

        let configuration = VZVirtualMachineConfiguration::new();
        configuration.setPlatform(&platform);
        configuration.setBootLoader(Some(&VZMacOSBootLoader::new()));
        configuration.setCPUCount(settings.cpus as usize);
        configuration.setMemorySize(settings.memory_bytes);

        let disk = VZDiskImageStorageDeviceAttachment::initWithURL_readOnly_error(
            VZDiskImageStorageDeviceAttachment::alloc(),
            &*file_url(&bundle.disk())?,
            false,
        )
        .map_err(|error| ns_error(&error))?;
        let block = VZVirtioBlockDeviceConfiguration::initWithAttachment(
            VZVirtioBlockDeviceConfiguration::alloc(),
            &disk,
        );
        configuration.setStorageDevices(&NSArray::from_retained_slice(&[block.into_super()]));

        let network = network_device(mac)?;
        configuration.setNetworkDevices(&NSArray::from_retained_slice(&[network.into_super()]));
        configuration.setEntropyDevices(&NSArray::from_retained_slice(&[
            VZVirtioEntropyDeviceConfiguration::new().into_super(),
        ]));
        configuration.setMemoryBalloonDevices(&NSArray::from_retained_slice(&[
            VZVirtioTraditionalMemoryBalloonDeviceConfiguration::new().into_super(),
        ]));
        configuration.setSocketDevices(&NSArray::from_retained_slice(&[
            VZVirtioSocketDeviceConfiguration::new().into_super(),
        ]));
        configuration.setKeyboards(&NSArray::from_retained_slice(&[
            VZMacKeyboardConfiguration::new().into_super(),
        ]));
        configuration.setPointingDevices(&NSArray::from_retained_slice(&[
            VZMacTrackpadConfiguration::new().into_super(),
        ]));
        if let Some(display) = settings.display {
            let graphics = graphics_device(display);
            configuration
                .setGraphicsDevices(&NSArray::from_retained_slice(&[graphics.into_super()]));
        }
        configuration
            .validateWithError()
            .map_err(|error| ns_error(&error))?;
        Ok(configuration)
    }
}

/// Check that a configuration can save and restore its state.
pub(crate) fn check_save_restore(configuration: &VZVirtualMachineConfiguration) -> Result<()> {
    // SAFETY: A validation call on a complete configuration.
    unsafe { configuration.validateSaveRestoreSupportWithError() }.map_err(|error| ns_error(&error))
}

fn network_device(mac: MacAddress) -> Result<Retained<VZVirtioNetworkDeviceConfiguration>> {
    let text = NSString::from_str(&mac.to_string());
    // SAFETY: Setters on new, unshared objects with values of the right type.
    unsafe {
        let address = VZMACAddress::initWithString(VZMACAddress::alloc(), &text)
            .ok_or_else(|| Error::InvalidMacAddress(mac.to_string()))?;
        let device = VZVirtioNetworkDeviceConfiguration::new();
        device.setMACAddress(&address);
        device.setAttachment(Some(&VZNATNetworkDeviceAttachment::new()));
        Ok(device)
    }
}

fn graphics_device(display: Display) -> Retained<VZMacGraphicsDeviceConfiguration> {
    // SAFETY: Setters on new, unshared objects with values of the right type.
    unsafe {
        let screen =
            VZMacGraphicsDisplayConfiguration::initWithWidthInPixels_heightInPixels_pixelsPerInch(
                VZMacGraphicsDisplayConfiguration::alloc(),
                display.width as isize,
                display.height as isize,
                display.pixels_per_inch as isize,
            );
        let device = VZMacGraphicsDeviceConfiguration::new();
        device.setDisplays(&NSArray::from_retained_slice(&[screen]));
        device
    }
}
