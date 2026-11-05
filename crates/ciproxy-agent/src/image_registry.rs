//! One owner coordinates selection, preparation holds, publication, and recovery.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageReport;

use crate::error::{Error, Result};
use crate::image_artifact::PendingImage;
use crate::image_store::ImageStore;
use crate::image_verification::CheckedBundle;

/// A preparation hold keeps its selected image until its last reference ends.
#[cfg_attr(test, unimock::unimock(api = ImageLeaseMock))]
pub trait ImageLease: Send + Sync {
    /// Return the selected image identity.
    fn image_id(&self) -> ImageId;
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
        let image_id = state
            .catalog
            .as_ref()
            .and_then(|report| report.newest.clone())
            .ok_or(Error::NoImage)?;
        *state.held.entry(image_id.clone()).or_default() += 1;
        Ok(Arc::new(HeldImage {
            image_id,
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
        let image_id = bundle.image_id().clone();
        self.store.publish(bundle)?;
        let catalog = state.catalog.as_mut().ok_or(Error::ImageOwnership)?;
        if !catalog.available.contains(&image_id) {
            catalog.available.push(image_id.clone());
        }
        catalog.newest = Some(image_id);
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
            catalog.available.retain(|id| keep.contains(id));
        }
        self.store.reclaim(&keep, &state.pending)
    }
}

struct HeldImage {
    image_id: ImageId,
    owner: Arc<ImageRegistry>,
}

impl ImageLease for HeldImage {
    fn image_id(&self) -> ImageId {
        self.image_id.clone()
    }
}

impl Drop for HeldImage {
    fn drop(&mut self) {
        let mut state = self.owner.state();
        if let Some(count) = state.held.get_mut(&self.image_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.held.remove(&self.image_id);
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
