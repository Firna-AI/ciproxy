//! Apple's macOS restore images (`.ipsw`).

use std::path::Path;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::NSError;
use objc2_virtualization::VZMacOSRestoreImage;

use super::{Once, Sendable, file_url, ns_error, receive};
use crate::error::{Error, Result};
use crate::settings::Limits;

/// A macOS restore image, either in Apple's catalogue or in a local file.
pub struct RestoreImage {
    image: Sendable<Retained<VZMacOSRestoreImage>>,
}

/// What the newest Mac platform of a restore image needs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Requirements {
    /// The `VZMacHardwareModel.dataRepresentation` to install with.
    pub hardware_model: Vec<u8>,
    /// The smallest supported CPU count.
    pub min_cpus: u32,
    /// The smallest supported memory size in bytes.
    pub min_memory_bytes: u64,
}

impl RestoreImage {
    /// Ask Apple's catalogue for the newest image that this Mac supports.
    ///
    /// The image is on the network. Download [`RestoreImage::url`] to a
    /// file, then [`RestoreImage::load`] the file to install from it.
    pub async fn latest() -> Result<Self> {
        let receiver = {
            let (block, receiver) = image_block();
            // SAFETY: The completion block has the documented type. The
            // framework copies the block, so it can drop here.
            unsafe { VZMacOSRestoreImage::fetchLatestSupportedWithCompletionHandler(&block) };
            receiver
        };
        receive(receiver).await
    }

    /// Load a restore image from a local `.ipsw` file.
    pub async fn load(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Err(Error::InvalidSettings(format!(
                "{} is not a restore image file",
                path.display()
            )));
        }
        let receiver = {
            let url = file_url(path)?;
            let (block, receiver) = image_block();
            // SAFETY: A local file URL, which the method needs, and a
            // completion block of the documented type.
            unsafe { VZMacOSRestoreImage::loadFileURL_completionHandler(&url, &block) };
            receiver
        };
        receive(receiver).await
    }

    /// The URL of the image: an `https` URL from the catalogue, or a
    /// `file` URL after [`RestoreImage::load`].
    pub fn url(&self) -> String {
        // SAFETY: Property reads of an immutable object.
        unsafe { self.image.0.URL().absoluteString() }
            .map(|url| url.to_string())
            .unwrap_or_default()
    }

    /// The build version, such as `25G84`.
    pub fn build_version(&self) -> String {
        // SAFETY: A property read of an immutable object.
        unsafe { self.image.0.buildVersion() }.to_string()
    }

    /// The macOS version as major, minor, and patch.
    pub fn os_version(&self) -> (isize, isize, isize) {
        // SAFETY: A property read of an immutable object.
        let version = unsafe { self.image.0.operatingSystemVersion() };
        (
            version.majorVersion,
            version.minorVersion,
            version.patchVersion,
        )
    }

    /// What this Mac needs to install the image, or `None` when this Mac
    /// cannot run it.
    pub fn requirements(&self) -> Option<Requirements> {
        // SAFETY: Property reads of immutable objects.
        unsafe {
            let configuration = self.image.0.mostFeaturefulSupportedConfiguration()?;
            let model = configuration.hardwareModel();
            if !model.isSupported() {
                return None;
            }
            Some(Requirements {
                hardware_model: model.dataRepresentation().to_vec(),
                min_cpus: u32::try_from(configuration.minimumSupportedCPUCount())
                    .unwrap_or(u32::MAX),
                min_memory_bytes: configuration.minimumSupportedMemorySize(),
            })
        }
    }

    pub(crate) fn file_url(&self) -> Retained<objc2_foundation::NSURL> {
        // SAFETY: A property read of an immutable object.
        unsafe { self.image.0.URL() }
    }
}

impl Requirements {
    /// The guest needs as limits, with no upper bound.
    pub fn limits(&self) -> Limits {
        Limits {
            min_cpus: self.min_cpus,
            max_cpus: u32::MAX,
            min_memory_bytes: self.min_memory_bytes,
            max_memory_bytes: u64::MAX,
        }
    }
}

type ImageBlock = RcBlock<dyn Fn(*mut VZMacOSRestoreImage, *mut NSError)>;

fn image_block() -> (
    ImageBlock,
    futures_channel::oneshot::Receiver<Result<RestoreImage>>,
) {
    let (once, receiver) = Once::new();
    let block = RcBlock::new(
        move |image: *mut VZMacOSRestoreImage, error: *mut NSError| {
            // SAFETY: The framework passes null or valid objects.
            let result = match unsafe { (Retained::retain(image), error.as_ref()) } {
                (Some(image), _) => Ok(RestoreImage {
                    image: Sendable(image),
                }),
                (None, Some(error)) => Err(ns_error(error)),
                (None, None) => Err(Error::NoResult),
            };
            once.send(result);
        },
    );
    (block, receiver)
}
