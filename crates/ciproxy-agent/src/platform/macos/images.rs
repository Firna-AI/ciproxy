//! The macOS image store on the area volume.
//!
//! `images/index.json` lists the published images. A download goes to
//! `images/NAME.tar`, extracts into `images/NAME.pending/`, and a rename
//! publishes `images/NAME/`. Recovery removes every pending part.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::{ImageDownload, ImageMetadata, ImageReport};
use ciproxy_protocol::platform::Platform;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use super::host::NewestImage;
use crate::boundaries::{ImageLease, Images};
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::hardware::{ImageNeeds, MacosVersion};
use crate::macos::image_bundle::{ImageIndex, IndexedImage, folder_name};
use crate::macos::image_extract::extract;
use crate::persistence::{atomic_private_write, private_directory};

pub(crate) struct MacImages {
    inner: Arc<Inner>,
}

struct Inner {
    folder: PathBuf,
    client: reqwest::Client,
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
                client,
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
        if self
            .inner
            .state()
            .index
            .images
            .iter()
            .any(|known| known.image_id == image.image_id)
        {
            return Ok(());
        }
        let name = folder_name(&image.image_id);
        let archive = self.inner.folder.join(format!("{name}.tar"));
        let pending = self.inner.folder.join(format!("{name}.pending"));
        self.download(image, &archive).await?;
        let root = image.root_disk_bytes;
        let (archive_path, pending_path) = (archive.clone(), pending.clone());
        let manifest =
            tokio::task::spawn_blocking(move || extract(&archive_path, &pending_path, root))
                .await
                .context("join image extraction")??;
        std::fs::remove_file(&archive).context("remove the image download")?;
        let target = self.inner.folder.join(&name);
        remove_folder(&target)?;
        std::fs::rename(&pending, &target).context("publish the image")?;
        let mut state = self.inner.state();
        state.index.publish(IndexedImage {
            image_id: image.image_id.clone(),
            root_disk_bytes: root,
            minimum_host_macos: manifest.minimum_host_macos,
            free_job_bytes: manifest.free_job_bytes,
        });
        self.inner.reclaim(&mut state)
    }
}

impl MacImages {
    /// Stream the archive to disk, then check its byte count and SHA-256.
    async fn download(&self, image: &ImageDownload, archive: &PathBuf) -> Result<()> {
        let url = image.download_url.expose_secret();
        if !url.starts_with("https://") {
            return Err(Error::HttpsRequired);
        }
        let response = self
            .inner
            .client
            .get(url)
            .send()
            .await
            .context("download image")?
            .error_for_status()
            .context("image download response")?;
        let mut file = tokio::fs::File::create(archive)
            .await
            .context("create download")?;
        let mut stream = response.bytes_stream();
        let (mut digest, mut bytes) = (Sha256::new(), 0_u64);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("receive image bytes")?;
            bytes = bytes.saturating_add(chunk.len() as u64);
            if bytes > image.size_bytes {
                return Err(Error::ImageVerification);
            }
            digest.update(&chunk);
            file.write_all(&chunk).await.context("write image bytes")?;
        }
        file.sync_all().await.context("sync the image download")?;
        let matches = hex::encode(digest.finalize()).eq_ignore_ascii_case(&image.sha256);
        if bytes != image.size_bytes || !matches {
            return Err(Error::ImageVerification);
        }
        Ok(())
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
