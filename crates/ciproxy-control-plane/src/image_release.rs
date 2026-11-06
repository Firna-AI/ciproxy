//! Resolve one pinned published image through authenticated Cloud CDN reads.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::secret::Secret;
use reqwest::Client;
use serde::Deserialize;

use crate::error::{ErrorResultExt, Result};
use crate::image_signing::ImageSigning;

/// Current published release facts, with no guessed root dimensions.
pub struct ImageRelease {
    /// Immutable artifact identity, hash, and download byte count.
    pub download: ImageDownload,
    /// Exact released root filesystem length.
    pub root_disk_bytes: u64,
}

/// Authenticated release discovery used before HTTP readiness.
#[async_trait]
pub trait ImageReleases: Send + Sync {
    /// Fetch the pinned manifest and its same-directory bundle metadata.
    async fn current(&self) -> Result<ImageRelease>;
}

/// CDN URLs use the existing mounted key and selected immutable release path.
pub struct CdnReleases {
    /// Redirect-disabled hosted HTTP client.
    pub http: Client,
    /// Existing public /images URL prefix.
    pub prefix: String,
    /// One immutable relative manifest path from operator configuration.
    pub manifest_path: String,
    /// Selected CDN signing key.
    pub signing: Arc<dyn ImageSigning>,
    /// Trusted receipt time for bounded URL signatures.
    pub clock: Arc<dyn Runtime>,
}

#[derive(Deserialize)]
struct Manifest {
    image_id: ImageId,
    root_disk_bytes: u64,
}
#[derive(Deserialize)]
struct Downloads {
    firecracker: Artifact,
}
#[derive(Deserialize)]
struct Artifact {
    file: String,
    sha256: String,
    size_bytes: u64,
}

impl CdnReleases {
    async fn read<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!(
            "{}/{}",
            self.prefix.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let signed = self.signing.sign(&url, self.clock.now())?;
        self.http
            .get(signed.expose_secret())
            .send()
            .await
            .internal()?
            .error_for_status()
            .internal()?
            .json()
            .await
            .internal()
    }
}

#[async_trait]
impl ImageReleases for CdnReleases {
    async fn current(&self) -> Result<ImageRelease> {
        let manifest: Manifest = self.read(&self.manifest_path).await?;
        let directory = self
            .manifest_path
            .rsplit_once('/')
            .map_or("", |(directory, _)| directory);
        let downloads: Downloads = self.read(&format!("{directory}/downloads.json")).await?;
        let download = ImageDownload {
            image_id: manifest.image_id,
            download_url: Secret::new(format!(
                "{}/{}/{}",
                self.prefix.trim_end_matches('/'),
                directory,
                downloads.firecracker.file
            )),
            sha256: downloads.firecracker.sha256,
            size_bytes: downloads.firecracker.size_bytes,
            root_disk_bytes: manifest.root_disk_bytes,
        };
        Ok(ImageRelease {
            download,
            root_disk_bytes: manifest.root_disk_bytes,
        })
    }
}
