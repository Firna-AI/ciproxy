//! Resolve one pinned published image release through signed reads.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use reqwest::{Client, ClientBuilder, redirect};
use serde::Deserialize;

use crate::error::{Error, ErrorResultExt, Result};
use crate::image_signing::ImageSigning;

/// The most bytes that one `manifest.json` or `downloads.json` may hold.
const METADATA_LIMIT_BYTES: usize = 1024 * 1024;
const METADATA_LIMIT: u64 = METADATA_LIMIT_BYTES as u64;

/// The client policy of every signed release read, with this request limit.
///
/// HTTP/1 only, no redirects, and a 5-second connect limit. Tests build on
/// it and add only a trust root or a proxy bypass.
pub fn release_client(limit: Duration) -> ClientBuilder {
    Client::builder()
        .http1_only()
        .redirect(redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(limit)
}

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
    /// The platform of the VMs that start from this release. It selects the
    /// archive in `downloads.json`: `firecracker` for Linux, `macos` for macOS.
    pub platform: Platform,
}

#[derive(Deserialize)]
struct Manifest {
    image_id: ImageId,
    root_disk_bytes: u64,
}
/// The typed Linux form refuses a second `firecracker` entry, as it always did.
#[derive(Deserialize)]
struct LinuxDownloads {
    firecracker: Artifact,
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
        let mut response = without_url(self.http.get(signed.expose_secret()).send().await)?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::ReleaseStatus(status.as_u16()));
        }
        if response
            .content_length()
            .is_some_and(|length| length > METADATA_LIMIT)
        {
            return Err(Error::ReleaseTooLarge);
        }
        // Count the streamed bytes too: a store need not send Content-Length.
        let mut body = Vec::new();
        while let Some(chunk) = without_url(response.chunk().await)? {
            if body.len() + chunk.len() > METADATA_LIMIT_BYTES {
                return Err(Error::ReleaseTooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&body).internal()
    }
}

/// Keep the HTTP failure, but drop its signed URL, which grants a read.
fn without_url<T>(result: reqwest::Result<T>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(error.without_url()).internal(),
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
        let path = format!("{directory}/downloads.json");
        let artifact = match self.platform {
            Platform::LinuxX64 => self.read::<LinuxDownloads>(&path).await?.firecracker,
            Platform::MacosArm64 => {
                // Other keys stay unread, so one bad entry cannot hide this artifact.
                let mut downloads: HashMap<String, serde_json::Value> = self.read(&path).await?;
                serde_json::from_value(
                    downloads
                        .remove("macos")
                        .ok_or(Error::MissingArtifact("macos"))?,
                )
                .internal()?
            }
        };
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
