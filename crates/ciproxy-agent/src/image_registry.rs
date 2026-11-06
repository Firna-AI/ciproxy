//! One owner coordinates selection, preparation holds, publication, and recovery.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageMetadata, ImageReport};

use crate::error::{Error, Result};
use crate::image_artifact::PendingImage;
use crate::image_store::ImageStore;
use crate::image_verification::CheckedBundle;

/// A preparation hold keeps its selected image until its last reference ends.
#[cfg_attr(test, unimock::unimock(api = ImageLeaseMock))]
pub trait ImageLease: Send + Sync {
    /// Return the exact selected release and its logical root length.
    fn metadata(&self) -> ImageMetadata;
}

#[derive(Default)]
struct RegistryState {
    catalog: Option<ImageReport>,
    active: bool,
    held: HashMap<ImageId, usize>,
    pending: Vec<PendingImage>,
}

pub(crate) struct ImageRegistry {
    store: Arc<dyn ImageStore>,
    state: Mutex<RegistryState>,
}

impl ImageRegistry {
    pub(crate) fn new(store: Arc<dyn ImageStore>) -> Self {
        Self {
            store,
            state: Mutex::new(RegistryState::default()),
        }
    }

    pub(crate) fn store(&self) -> &dyn ImageStore {
        self.store.as_ref()
    }

    fn state(&self) -> MutexGuard<'_, RegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    pub(crate) fn report(&self) -> Result<ImageReport> {
        let state = self.state();
        match &state.catalog {
            Some(report) => Ok(report.clone()),
            None => self.store.report(),
        }
    }

    pub(crate) fn is_newest(&self, image_id: &ImageId) -> bool {
        let state = self.state();
        state.active
            && state
                .catalog
                .as_ref()
                .is_some_and(|catalog| catalog.newest.as_ref() == Some(image_id))
    }

    pub(crate) fn recover(&self) -> Result<()> {
        let mut state = self.state();
        if state.active {
            return Ok(());
        }
        self.store.activate()?;
        state.catalog = Some(self.store.report()?);
        state.active = true;
        self.reclaim(&mut state)?;
        self.store.admit(0)
    }

    pub(crate) fn hold_newest(self: &Arc<Self>) -> Result<Arc<dyn ImageLease>> {
        let mut state = self.state();
        if !state.active {
            return Err(Error::ImageOwnership);
        }
        let metadata = state
            .catalog
            .as_ref()
            .and_then(|report| report.selected().cloned())
            .ok_or(Error::NoImage)?;
        *state.held.entry(metadata.image_id.clone()).or_default() += 1;
        Ok(Arc::new(HeldImage {
            metadata,
            owner: self.clone(),
        }))
    }

    pub(crate) fn begin(self: &Arc<Self>, archive_bytes: u64) -> Result<PendingDownload> {
        let mut state = self.state();
        if !state.active {
            return Err(Error::ImageOwnership);
        }
        self.reclaim(&mut state)?;
        self.store.admit(archive_bytes.saturating_add(128 * 1024))?;
        let image = self.store.begin()?;
        state.pending.push(image.clone());
        Ok(PendingDownload {
            image,
            owner: self.clone(),
        })
    }

    pub(crate) fn publish(&self, bundle: CheckedBundle) -> Result<()> {
        let mut state = self.state();
        let metadata = bundle.metadata().clone();
        self.store.publish(bundle)?;
        let catalog = state.catalog.as_mut().ok_or(Error::ImageOwnership)?;
        if !catalog.available.contains(&metadata) {
            catalog.available.push(metadata.clone());
        }
        catalog.newest = Some(metadata.image_id);
        if let Err(error) = self.reclaim(&mut state) {
            tracing::warn!(error = %error, "image published; reclaim will retry");
        }
        Ok(())
    }

    pub(crate) fn purge(&self) -> Result<()> {
        let mut state = self.state();
        if !state.active || !state.held.is_empty() || !state.pending.is_empty() {
            return Err(Error::ImageOwnership);
        }
        state.catalog = Some(ImageReport {
            available: Vec::new(),
            newest: None,
        });
        self.reclaim(&mut state)
    }

    fn reclaim(&self, state: &mut RegistryState) -> Result<()> {
        let mut keep = state.held.keys().cloned().collect::<Vec<_>>();
        if let Some(image_id) = state
            .catalog
            .as_ref()
            .and_then(|report| report.newest.clone())
            && !keep.contains(&image_id)
        {
            keep.push(image_id);
        }
        keep.sort_by(|left, right| left.0.cmp(&right.0));
        if let Some(catalog) = &mut state.catalog {
            catalog
                .available
                .retain(|image| keep.contains(&image.image_id));
        }
        self.store.reclaim(&keep, &state.pending)
    }
}

struct HeldImage {
    metadata: ImageMetadata,
    owner: Arc<ImageRegistry>,
}

impl ImageLease for HeldImage {
    fn metadata(&self) -> ImageMetadata {
        self.metadata.clone()
    }
}

impl Drop for HeldImage {
    fn drop(&mut self) {
        let mut state = self.owner.state();
        if let Some(count) = state.held.get_mut(&self.metadata.image_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.held.remove(&self.metadata.image_id);
            }
        }
        if let Err(error) = self.owner.reclaim(&mut state) {
            tracing::warn!(error = %error, "image hold ended; reclaim will retry");
        }
    }
}

pub(crate) struct PendingDownload {
    image: PendingImage,
    owner: Arc<ImageRegistry>,
}

impl PendingDownload {
    pub(crate) fn image(&self) -> &PendingImage {
        &self.image
    }
}

impl Drop for PendingDownload {
    fn drop(&mut self) {
        let mut state = self.owner.state();
        state.pending.retain(|pending| *pending != self.image);
        if let Err(error) = self.owner.reclaim(&mut state) {
            tracing::warn!(error = %error, "image fetch ended; reclaim will retry");
        }
    }
}
