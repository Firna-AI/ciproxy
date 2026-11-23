//! The macOS image store on the area volume.
//!
//! `images/index.json` lists the published images. A download writes
//! `images/NAME.tar.partial`, which becomes `images/NAME.tar` after its
//! check. The archive extracts into `images/NAME.pending/`, and a rename
//! publishes `images/NAME/`. Recovery removes every pending part, and keeps
//! a partial download that its sidecar describes. `import-image` puts a
//! local archive at `images/NAME.tar` instead of the download, and then
//! publishes it in the same way.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageDownload, ImageMetadata, ImageReport};
use ciproxy_protocol::platform::Platform;

use super::host::NewestImage;
use crate::boundaries::{ImageLease, Images};
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::hardware::{ImageNeeds, MacosVersion};
use crate::macos::image_bundle::{ImageIndex, IndexedImage, folder_name};
use crate::macos::image_extract::extract;
use crate::persistence::{atomic_private_write, private_directory};
use crate::resumable_download::{self, ResumableDownload, Retry};

pub(crate) struct MacImages {
    inner: Arc<Inner>,
}

struct Inner {
    folder: PathBuf,
    download: ResumableDownload,
    /// One download at a time, so that two commands never write the same
    /// partial file.
    fetching: tokio::sync::Mutex<()>,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    index: ImageIndex,
    /// Holds by image ID text.
    held: BTreeMap<String, usize>,
}

impl MacImages {
    pub(crate) fn new(area: PathBuf, client: reqwest::Client) -> Self {
        Self {
            inner: Arc::new(Inner {
                folder: area.join("images"),
                download: ResumableDownload::new(client, Retry::BROKEN_STREAM),
                fetching: tokio::sync::Mutex::new(()),
                state: Mutex::new(State::default()),
            }),
        }
    }
}

impl Inner {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    fn save(&self, index: &ImageIndex) -> Result<()> {
        let bytes = serde_json::to_vec(index).context("encode the image index")?;
        atomic_private_write(&self.folder.join("index.json"), &bytes)
    }

    /// Remove each image that is neither the newest nor held.
    fn reclaim(&self, state: &mut State) -> Result<()> {
        let held: Vec<ImageId> = state.held.keys().cloned().map(ImageId).collect();
        let removed = state.index.reclaim(&held);
        self.save(&state.index)?;
        for image in removed {
            remove_folder(&self.folder.join(folder_name(&image)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl Images for MacImages {
    fn report(&self) -> Result<ImageReport> {
        Ok(self.inner.state().index.report())
    }

    fn recover(&self) -> Result<()> {
        let folder = &self.inner.folder;
        private_directory(folder)?;
        resumable_download::recover(folder)?;
        for entry in std::fs::read_dir(folder).context("list images")? {
            let name = entry.context("read an image entry")?.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".pending") || name.ends_with(".tar") {
                let path = folder.join(name.as_ref());
                if path.is_dir() {
                    remove_folder(&path)?;
                } else {
                    std::fs::remove_file(&path).context("remove an image download")?;
                }
            }
        }
        let mut index: ImageIndex = match std::fs::read(folder.join("index.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes).context("decode the image index")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ImageIndex::default(),
            Err(error) => return Err(error).context("read the image index"),
        };
        index.images.retain(|image| {
            folder
                .join(folder_name(&image.image_id))
                .join("bundle")
                .is_dir()
        });
        if index.newest().is_none() {
            index.newest = index.images.last().map(|image| image.image_id.clone());
        }
        let mut state = self.inner.state();
        state.index = index;
        self.inner.reclaim(&mut state)
    }

    fn hold_newest(&self) -> Result<Arc<dyn ImageLease>> {
        let mut state = self.inner.state();
        let image = state.index.newest().ok_or(Error::NoImage)?;
        let metadata = ImageMetadata {
            image_id: image.image_id.clone(),
            root_disk_bytes: image.root_disk_bytes,
            free_job_bytes: Some(image.free_job_bytes),
        };
        *state.held.entry(metadata.image_id.0.clone()).or_default() += 1;
        Ok(Arc::new(HeldImage {
            metadata,
            owner: self.inner.clone(),
        }))
    }

    async fn fetch(&self, image: &ImageDownload) -> Result<()> {
        if image.platform != Platform::MacosArm64 {
            return Err(Error::PlatformMismatch);
        }
        let _fetching = self.inner.fetching.lock().await;
        if self.known(&image.image_id) {
            return Ok(());
        }
        if !image.download_url.expose_secret().starts_with("https://") {
            return Err(Error::HttpsRequired);
        }
        let archive = self.archive(&image.image_id);
        self.inner.download.fetch(image, &archive).await?;
        self.publish(&image.image_id, image.root_disk_bytes, &archive)
            .await
    }
}

impl MacImages {
    /// The index already lists this image.
    pub(super) fn known(&self, image: &ImageId) -> bool {
        self.inner
            .state()
            .index
            .images
            .iter()
            .any(|known| known.image_id == *image)
    }

    /// `images/NAME.tar`: where a checked archive waits for extraction.
    /// Recovery removes it.
    pub(super) fn archive(&self, image: &ImageId) -> PathBuf {
        self.inner
            .folder
            .join(format!("{}.tar", folder_name(image)))
    }

    /// Extract a checked archive, publish `images/NAME/`, and make it the
    /// newest image. The download and `import-image` both end here.
    pub(super) async fn publish(
        &self,
        image_id: &ImageId,
        root: u64,
        archive: &std::path::Path,
    ) -> Result<()> {
        let name = folder_name(image_id);
        let pending = self.inner.folder.join(format!("{name}.pending"));
        let (archive_path, pending_path) = (archive.to_path_buf(), pending.clone());
        let manifest =
            tokio::task::spawn_blocking(move || extract(&archive_path, &pending_path, root))
                .await
                .context("join image extraction")??;
        std::fs::remove_file(archive).context("remove the image download")?;
        let target = self.inner.folder.join(&name);
        remove_folder(&target)?;
        std::fs::rename(&pending, &target).context("publish the image")?;
        let mut state = self.inner.state();
        state.index.publish(IndexedImage {
            image_id: image_id.clone(),
            root_disk_bytes: root,
            minimum_host_macos: manifest.minimum_host_macos,
            free_job_bytes: manifest.free_job_bytes,
        });
        self.inner.reclaim(&mut state)
    }
}

impl NewestImage for MacImages {
    fn needs(&self) -> Option<ImageNeeds> {
        let state = self.inner.state();
        let image = state.index.newest()?;
        Some(ImageNeeds {
            minimum_host: MacosVersion::parse(&image.minimum_host_macos)?,
            hardware_model_supported: true,
        })
    }
}

/// An image stays on disk while a VM preparation holds it.
struct HeldImage {
    metadata: ImageMetadata,
    owner: Arc<Inner>,
}

impl ImageLease for HeldImage {
    fn metadata(&self) -> ImageMetadata {
        self.metadata.clone()
    }
}

impl Drop for HeldImage {
    fn drop(&mut self) {
        let mut state = self.owner.state();
        if let Some(count) = state.held.get_mut(&self.metadata.image_id.0) {
            *count -= 1;
            if *count == 0 {
                state.held.remove(&self.metadata.image_id.0);
            }
        }
        if let Err(error) = self.owner.reclaim(&mut state) {
            tracing::warn!(%error, "image reclaim will retry");
        }
    }
}

fn remove_folder(path: &std::path::Path) -> Result<()> {
    match std::fs::remove_dir_all(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).context("remove an image folder")
        }
        _ => Ok(()),
    }
}
