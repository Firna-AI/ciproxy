//! Image adapters constructed by the Linux application composition root.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;

use crate::boundaries::Images;
use crate::error::{ErrorResultExt, Result};
use crate::image_download::HttpsImageDownload;
use crate::image_space::{ImageBudget, ImageSpace, LinuxSpaceUsage};
use crate::image_store::LinuxImageStore;
use crate::images::ManagedImages;
use crate::layout::Layout;

pub(crate) struct ImageServices {
    pub(crate) images: Arc<dyn Images>,
    pub(crate) space: Arc<dyn ImageSpace>,
}

pub(crate) fn prepare(layout: &Layout, image_budget: u64) -> Result<ImageServices> {
    let space: Arc<dyn ImageSpace> = Arc::new(ImageBudget::new(
        image_budget,
        Arc::new(LinuxSpaceUsage {
            area: layout.data.clone(),
        }),
    ));
    let images: Arc<dyn Images> = Arc::new(ManagedImages::new(
        Arc::new(LinuxImageStore::new(layout.clone(), space.clone())),
        Arc::new(HttpsImageDownload {
            client: Client::builder()
                .https_only(true)
                .http1_only()
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(3600))
                .build()
                .context("configure image downloads")?,
        }),
    ));
    Ok(ImageServices { images, space })
}
