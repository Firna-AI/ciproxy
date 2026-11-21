//! Image adapters constructed by the Linux application composition root.

use std::sync::Arc;
use std::time::Duration;

use crate::boundaries::Images;
use crate::error::{ErrorResultExt, Result};
use crate::image_download::HttpsImageDownload;
use crate::image_space::{ImageBudget, ImageSpace, LinuxSpaceUsage};
use crate::image_store::LinuxImageStore;
use crate::images::ManagedImages;
use crate::layout::Layout;
use crate::runtime_ownership::{RuntimeClaims, RuntimeOwner};

mod client;

/// Two minutes tolerate temporary network pauses without leaving a stalled fetch open.
const IMAGE_READ_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) struct ImageServices {
    pub(crate) images: Arc<dyn Images>,
    pub(crate) space: Arc<dyn ImageSpace>,
}

pub(crate) fn prepare(
    layout: &Layout,
    image_budget: u64,
    claims: Arc<dyn RuntimeClaims>,
    owner: Option<Arc<dyn RuntimeOwner>>,
) -> Result<ImageServices> {
    let space: Arc<dyn ImageSpace> = Arc::new(ImageBudget::new(
        image_budget,
        Arc::new(LinuxSpaceUsage {
            layout: layout.clone(),
        }),
    ));
    let images: Arc<dyn Images> = Arc::new(ManagedImages::new(
        Arc::new(LinuxImageStore::new(
            layout.clone(),
            space.clone(),
            claims,
            owner,
        )),
        Arc::new(HttpsImageDownload {
            client: client::builder(Duration::from_secs(20), IMAGE_READ_TIMEOUT)
                .build()
                .context("configure image downloads")?,
        }),
    ));
    Ok(ImageServices { images, space })
}
