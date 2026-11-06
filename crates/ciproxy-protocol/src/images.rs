//! Image download metadata and the images available on one machine.

use serde::{Deserialize, Serialize};

use crate::identifiers::ImageId;
use crate::secret::Secret;

/// One published image that the agent can fetch and verify.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageDownload {
    /// The release that this download provides.
    pub image_id: ImageId,
    /// Download URL, including any signature, hidden from diagnostics.
    pub download_url: Secret,
    /// SHA-256 digest in hexadecimal for verification after download.
    pub sha256: String,
    /// Expected size of the download, in bytes.
    pub size_bytes: u64,
    /// Logical root length from this release's metadata.
    pub root_disk_bytes: u64,
}

/// Exact disk facts attached to one verified image release.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageMetadata {
    /// Image release identity.
    pub image_id: ImageId,
    /// Logical length of that release's verified root disk.
    pub root_disk_bytes: u64,
}

/// The machine's locally available image releases.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageReport {
    /// Images that the agent has downloaded and checked.
    pub available: Vec<ImageMetadata>,
    /// Newest available image, used for new VMs when one exists.
    pub newest: Option<ImageId>,
}

impl ImageReport {
    /// Resolve exact selected-release facts without guessing unknown metadata.
    pub fn selected(&self) -> Option<&ImageMetadata> {
        let newest = self.newest.as_ref()?;
        self.available
            .iter()
            .find(|image| image.image_id == *newest)
    }
}
