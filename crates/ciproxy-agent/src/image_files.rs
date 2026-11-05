//! Linux image cache uses an exclusive runtime owner and atomic file names.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use async_trait::async_trait;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageReport;
use fs2::FileExt;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::host_paths::CONTROL;
use crate::image_artifact::PendingImage;
use crate::image_store::{ImageStore, LinuxImageStore};
use crate::image_verification::{CheckedBundle, CheckedDownload};
use crate::persistence::{atomic_private_write, private_directory};

#[async_trait]
impl ImageStore for LinuxImageStore {
    fn report(&self) -> Result<ImageReport> {
        match fs::read(self.layout.data.join("images/index.json")) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).context("read image index")?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ImageReport {
                available: Vec::new(),
                newest: None,
            }),
            Err(error) => Err(error).context("read image index"),
        }
    }

    fn activate(&self) -> Result<()> {
        let mut owner = self
            .owner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if owner.is_some() {
            return Ok(());
        }
        private_directory(&self.layout.data)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(Path::new(CONTROL).join("images.lock"))
            .context("open image ownership lock")?;
        FileExt::try_lock_exclusive(&file).context("claim image cache ownership")?;
        private_directory(&self.layout.data.join("images"))?;
        *owner = Some(file);
        Ok(())
    }

    fn begin(&self) -> Result<PendingImage> {
        self.require_owner()?;
        let directory = self
            .layout
            .data
            .join("images")
            .join(format!("pending-{}", Uuid::new_v4().simple()));
        private_directory(&directory)?;
        Ok(PendingImage::new(directory))
    }

    async fn extract(&self, checked: CheckedDownload) -> Result<CheckedBundle> {
        self.require_owner()?;
        spawn_blocking(move || Self::unpack(checked))
            .await
            .context("extract image task")?
    }

    fn publish(&self, bundle: CheckedBundle) -> Result<()> {
        self.require_owner()?;
        let image_id = bundle.image_id().clone();
        let root = bundle.pending().directory().join("verified");
        let destination = self.layout.image(&image_id);
        if !destination.exists() {
            Self::sync_directory(&root)?;
            fs::rename(&root, &destination).context("publish checked image")?;
            Self::sync_directory(&self.layout.data.join("images"))?;
        }
        let mut catalog = self.report()?;
        if !catalog.available.contains(&image_id) {
            catalog.available.push(image_id.clone());
        }
        catalog.newest = Some(image_id);
        self.write_index(&catalog)
    }

    fn reclaim(&self, keep: &[ImageId], pending: &[PendingImage]) -> Result<()> {
        self.require_owner()?;
        let directory = self.layout.data.join("images");
        let mut catalog = self.report()?;
        catalog.available.retain(|image| keep.contains(image));
        if catalog
            .newest
            .as_ref()
            .is_some_and(|image| !keep.contains(image))
        {
            catalog.newest = None;
        }
        self.write_index(&catalog)?;
        let mut retained = keep
            .iter()
            .map(|image| self.layout.image(image))
            .collect::<Vec<_>>();
        retained.extend(pending.iter().map(|pending| pending.directory().to_owned()));
        retained.push(directory.join("index.json"));
        let entries = fs::read_dir(&directory)
            .context("list owned image cache")?
            .collect::<io::Result<Vec<_>>>()
            .context("read owned image entries")?;
        for entry in entries {
            if retained.contains(&entry.path()) {
                continue;
            }
            let kind = entry.file_type().context("inspect owned image entry")?;
            let garbage = directory.join(format!("garbage-{}", Uuid::new_v4().simple()));
            fs::rename(entry.path(), &garbage).context("retire image cache name")?;
            Self::sync_directory(&directory)?;
            if kind.is_dir() {
                fs::remove_dir_all(&garbage).context("reclaim image directory")?;
            } else {
                fs::remove_file(&garbage).context("reclaim image file")?;
            }
        }
        Self::sync_directory(&directory)
    }
}

impl LinuxImageStore {
    fn require_owner(&self) -> Result<()> {
        if self
            .owner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .is_none()
        {
            return Err(Error::ImageOwnership);
        }
        Ok(())
    }

    fn write_index(&self, catalog: &ImageReport) -> Result<()> {
        atomic_private_write(
            &self.layout.data.join("images/index.json"),
            &serde_json::to_vec(catalog).context("serialize image index")?,
        )
    }

    fn sync_directory(path: &Path) -> Result<()> {
        File::open(path)
            .context("open image directory")?
            .sync_all()
            .context("sync image directory")?;
        Ok(())
    }
}
