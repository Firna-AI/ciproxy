//! The only filesystem owner of downloaded images and interrupted artifacts.

use std::fs::File;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageReport;

use crate::error::Result;
use crate::image_artifact::PendingImage;
use crate::image_space::ImageSpace;
use crate::image_verification::{CheckedBundle, CheckedDownload};
use crate::layout::Layout;

#[cfg_attr(test, unimock::unimock(api = ImageStoreMock))]
#[async_trait]
pub(crate) trait ImageStore: Send + Sync {
    fn report(&self) -> Result<ImageReport>;
    fn activate(&self) -> Result<()>;
    fn begin(&self) -> Result<PendingImage>;
    fn admit(&self, bytes: u64) -> Result<()>;
    async fn extract(&self, checked: CheckedDownload) -> Result<CheckedBundle>;
    fn publish(&self, bundle: CheckedBundle) -> Result<()>;
    fn reclaim(&self, keep: &[ImageId], pending: &[PendingImage]) -> Result<()>;
}

pub(crate) struct LinuxImageStore {
    pub(crate) layout: Layout,
    pub(crate) owner: Mutex<Option<File>>,
    pub(crate) space: Arc<dyn ImageSpace>,
}

impl LinuxImageStore {
    pub(crate) fn new(layout: Layout, space: Arc<dyn ImageSpace>) -> Self {
        Self {
            layout,
            owner: Mutex::new(None),
            space,
        }
    }
}
