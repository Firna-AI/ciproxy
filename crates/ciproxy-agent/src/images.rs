//! HTTPS image downloads become usable only after byte-count and SHA-256 checks.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::images::{ImageDownload, ImageReport};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::boundaries::Images;
use crate::error::{Error, ErrorResultExt, Result};
use crate::image_verification::{DownloadDigest, ImageMember};
use crate::layout::Layout;
use crate::persistence::{atomic_private_write, private_directory};

#[derive(Clone)]
pub(crate) struct LinuxImages {
    pub(crate) layout: Layout,
    pub(crate) client: Client,
    pub(crate) operation: Arc<Mutex<()>>,
}

#[async_trait]
impl Images for LinuxImages {
    fn report(&self) -> Result<ImageReport> {
        match fs::read(self.layout.data.join("images/index.json")) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).context("read verified image index")?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ImageReport {
                available: Vec::new(),
                newest: None,
            }),
            Err(error) => Err(error).context("read image index"),
        }
    }

    async fn fetch(&self, image: &ImageDownload) -> Result<()> {
        let _operation = self.operation.lock().await;
        let url = image.download_url.expose_secret();
        if !url.starts_with("https://") {
            return Err(Error::HttpsRequired);
        }
        let directory = self.layout.data.join("images");
        private_directory(&directory)?;
        let pending = directory.join(format!("pending-{}", Uuid::new_v4().simple()));
        private_directory(&pending)?;
        let result = self.download_and_publish(image, &pending).await;
        if let Err(error) = fs::remove_dir_all(&pending)
            && error.kind() != io::ErrorKind::NotFound
        {
            tracing::warn!(error = %error, "cannot remove temporary image download");
        }
        result
    }
}

impl LinuxImages {
    async fn download_and_publish(&self, image: &ImageDownload, pending: &Path) -> Result<()> {
        let archive_path = pending.join("image.tar.gz");
        let mut archive = File::create(&archive_path)
            .await
            .context("create temporary image download")?;
        let response = self
            .client
            .get(image.download_url.expose_secret())
            .send()
            .await
            .context("download image")?
            .error_for_status()
            .context("image download response")?;
        let mut stream = response.bytes_stream();
        let mut digest = DownloadDigest::default();
        let mut bytes = 0_u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("receive image bytes")?;
            bytes = bytes.saturating_add(chunk.len() as u64);
            if bytes > image.size_bytes {
                return Err(Error::ImageVerification);
            }
            archive
                .write_all(&chunk)
                .await
                .context("write image download")?;
            digest.update(&chunk);
        }
        digest.finish(image)?;
        archive.sync_all().await.context("sync verified download")?;
        let publisher = self.clone();
        let pending = pending.to_owned();
        let image = image.clone();
        spawn_blocking(move || publisher.publish(&image, &pending))
            .await
            .context("publish checked image task")?
    }

    fn publish(&self, image: &ImageDownload, pending: &Path) -> Result<()> {
        let archive_path = pending.join("image.tar.gz");
        let root = pending.join("verified");
        private_directory(&root)?;
        let archive = fs::File::open(&archive_path).context("open checked image")?;
        let mut tar = tar::Archive::new(GzDecoder::new(archive));
        let mut members = Vec::new();
        for entry in tar.entries().context("read image bundle")? {
            let mut entry = entry.context("read image member")?;
            let member = ImageMember::parse(
                &entry.path().context("read image member path")?,
                entry.header().entry_type().is_file(),
            )?;
            if members.contains(&member) {
                return Err(Error::ImageLayout);
            }
            let destination = root.join(member.name());
            let mut file = fs::File::create(&destination).context("create verified image file")?;
            io::copy(&mut entry, &mut file).context("extract checked image file")?;
            if file.metadata().context("inspect image file")?.len() == 0 {
                return Err(Error::ImageLayout);
            }
            file.sync_all().context("sync image file")?;
            fs::set_permissions(&destination, fs::Permissions::from_mode(0o400))
                .context("protect verified image")?;
            members.push(member);
        }
        if members.len() != 2 {
            return Err(Error::ImageLayout);
        }
        let destination = self.layout.image(&image.image_id);
        if !destination.exists() {
            fs::rename(&root, &destination).context("publish verified image")?;
        }
        let mut report = self.report()?;
        if !report.available.contains(&image.image_id) {
            report.available.push(image.image_id.clone());
        }
        report.newest = Some(image.image_id.clone());
        atomic_private_write(
            &self.layout.data.join("images/index.json"),
            &serde_json::to_vec(&report).context("serialize image index")?,
        )
    }
}
