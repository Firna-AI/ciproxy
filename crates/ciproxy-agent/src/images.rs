//! Image orchestration requires checked values and explicit preparation holds.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::images::{ImageDownload, ImageReport};
use tokio::sync::Mutex;

use crate::boundaries::Images;
use crate::error::Result;
use crate::image_download::Download;
use crate::image_registry::{ImageLease, ImageRegistry};
use crate::image_store::ImageStore;

pub(crate) struct ManagedImages {
    registry: Arc<ImageRegistry>,
    download: Arc<dyn Download>,
    fetch: Mutex<()>,
}

impl ManagedImages {
    pub(crate) fn new(store: Arc<dyn ImageStore>, download: Arc<dyn Download>) -> Self {
        Self {
            registry: Arc::new(ImageRegistry::new(store)),
            download,
            fetch: Mutex::new(()),
        }
    }
}

#[async_trait]
impl Images for ManagedImages {
    fn report(&self) -> Result<ImageReport> {
        self.registry.report()
    }

    fn recover(&self) -> Result<()> {
        self.registry.recover()
    }

    fn hold_newest(&self) -> Result<Arc<dyn ImageLease>> {
        self.registry.hold_newest()
    }

    async fn fetch(&self, image: &ImageDownload) -> Result<()> {
        let _fetch = self.fetch.lock().await;
        if self.registry.is_newest(&image.image_id) {
            return Ok(());
        }
        let pending = self.registry.begin(image.size_bytes)?;
        let digest = self.download.download(image, pending.image()).await?;
        let checked = digest.finish(image, pending.image().clone())?;
        let bundle = self.registry.store().extract(checked).await?;
        self.registry.publish(bundle)
    }
}

#[cfg(test)]
#[path = "_tests_/images_tests.rs"]
mod images_tests;

#[cfg(test)]
#[path = "_tests_/image_budget_tests.rs"]
mod image_budget_tests;
