//! Resolve one pinned published image release through signed reads.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use reqwest::Client;
use serde::Deserialize;

use crate::error::{Error, ErrorResultExt, Result};
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

/// Reads one pinned release of one platform under a URL prefix.
///
/// The Linux release uses the CDN key and the `firecracker` artifact. The
/// macOS release uses a presigned R2 URL and the `macos` artifact.
pub struct SignedReleases {
    /// Redirect-disabled hosted HTTP client.
    pub http: Client,
    /// URL prefix of the release store, such as the public /images prefix.
    pub prefix: String,
    /// One immutable relative manifest path from operator configuration.
    pub manifest_path: String,
    /// Signs each read of a release file.
    pub signing: Arc<dyn ImageSigning>,
    /// Trusted receipt time for bounded URL signatures.
    pub clock: Arc<dyn Runtime>,
    /// The platform of the VMs that start from this release.
    pub platform: Platform,
    /// The key of the downloaded artifact in `downloads.json`.
    pub artifact: &'static str,
}

#[derive(Deserialize)]
struct Manifest {
    image_id: ImageId,
    root_disk_bytes: u64,
}
#[derive(Deserialize)]
struct Artifact {
    file: String,
    sha256: String,
    size_bytes: u64,
}

impl SignedReleases {
    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.prefix.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn read<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let signed = self.signing.sign(&self.url(path), self.clock.now())?;
        let response = self
            .http
            .get(signed.expose_secret())
            .send()
            .await
            .internal()?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::ReleaseStatus(status.as_u16()));
        }
        response.json().await.internal()
    }
}

#[async_trait]
impl ImageReleases for SignedReleases {
    async fn current(&self) -> Result<ImageRelease> {
        let manifest: Manifest = self.read(&self.manifest_path).await?;
        let directory = self
            .manifest_path
            .rsplit_once('/')
            .map_or("", |(directory, _)| directory);
        // Other keys stay unread, so one bad entry cannot hide this artifact.
        let mut downloads: HashMap<String, serde_json::Value> =
            self.read(&format!("{directory}/downloads.json")).await?;
        let artifact: Artifact = serde_json::from_value(
            downloads
                .remove(self.artifact)
                .ok_or(Error::MissingArtifact(self.artifact))?,
        )
        .internal()?;
        let download = ImageDownload {
            image_id: manifest.image_id,
            platform: self.platform,
            download_url: Secret::new(self.url(&format!("{directory}/{}", artifact.file))),
            sha256: artifact.sha256,
            size_bytes: artifact.size_bytes,
            root_disk_bytes: manifest.root_disk_bytes,
        };
        Ok(ImageRelease {
            download,
            root_disk_bytes: manifest.root_disk_bytes,
        })
    }
}
