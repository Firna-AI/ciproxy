//! The optional macOS release, read from the private R2 image store.

use std::env;
use std::sync::Arc;
use std::time::Duration;

use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use reqwest::Client;

use crate::error::Result;
use crate::image_release::{ImageReleases, SignedReleases};
use crate::image_signing::ImageSigning;
use crate::presigned_signing::{PresignedSigning, StoreKey};

/// The S3 region name of every R2 bucket.
const R2_REGION: &str = "auto";
/// Validity of a presigned read of `manifest.json` or `downloads.json`.
const RELEASE_READ_EXPIRY: Duration = Duration::from_secs(5 * 60);
/// Validity of the presigned archive URL in each `FetchImage`.
const DOWNLOAD_EXPIRY: Duration = Duration::from_secs(2 * 60 * 60);
const KEY_SETTING: &str = "R2_IMAGES_KEY";

/// The three settings of the macOS release. An unset or empty variable is absent.
#[derive(Clone, Default)]
pub struct MacosImageSettings {
    /// R2 endpoint and bucket in path style, from `MACOS_IMAGE_URL_PREFIX`.
    pub url_prefix: Option<String>,
    /// Pinned manifest path under the prefix, from `MACOS_IMAGE_MANIFEST_PATH`.
    pub manifest_path: Option<String>,
    /// Read key of the bucket as JSON, from `R2_IMAGES_KEY`.
    pub key: Option<Secret>,
}

struct Source<'a> {
    url_prefix: &'a str,
    manifest_path: &'a str,
    key: StoreKey,
}

impl MacosImageSettings {
    /// Read the three variables. A missing variable never stops the process.
    pub fn from_env() -> Self {
        let read = |name| env::var(name).ok().filter(|value| !value.trim().is_empty());
        Self {
            url_prefix: read("MACOS_IMAGE_URL_PREFIX"),
            manifest_path: read("MACOS_IMAGE_MANIFEST_PATH"),
            key: read(KEY_SETTING).map(Secret::new),
        }
    }

    /// The signer of macOS `FetchImage` URLs, or nothing while macOS images are off.
    pub fn download_signing(&self) -> Option<Arc<dyn ImageSigning>> {
        let source = self.source().ok().flatten()?;
        Some(Arc::new(PresignedSigning::new(
            source.key,
            R2_REGION,
            DOWNLOAD_EXPIRY,
        )))
    }

    fn missing(&self) -> Vec<&'static str> {
        [
            ("MACOS_IMAGE_URL_PREFIX", self.url_prefix.is_none()),
            ("MACOS_IMAGE_MANIFEST_PATH", self.manifest_path.is_none()),
            (KEY_SETTING, self.key.is_none()),
        ]
        .into_iter()
        .filter_map(|(name, missing)| missing.then_some(name))
        .collect()
    }

    fn source(&self) -> Result<Option<Source<'_>>> {
        let (Some(url_prefix), Some(manifest_path), Some(key)) =
            (&self.url_prefix, &self.manifest_path, &self.key)
        else {
            return Ok(None);
        };
        Ok(Some(Source {
            url_prefix: url_prefix.trim(),
            manifest_path: manifest_path.trim(),
            key: StoreKey::from_json(KEY_SETTING, key.expose_secret())?,
        }))
    }
}

/// Read the pinned macOS release when all three settings are set.
///
/// Otherwise, and after a failed read, log why macOS images are off and give
/// nothing. Linux work never waits for or fails with this release.
pub async fn read_macos_release(
    settings: &MacosImageSettings,
    http: Client,
    clock: Arc<dyn Runtime>,
) -> Option<ImageDownload> {
    let source = match settings.source() {
        Ok(Some(source)) => source,
        Ok(None) => {
            let missing = settings.missing().join(", ");
            tracing::info!(%missing, "macOS images are off: a setting is missing");
            return None;
        }
        Err(error) => {
            tracing::warn!(%error, "macOS images are off: the R2 key is malformed");
            return None;
        }
    };
    let releases = SignedReleases {
        http,
        prefix: source.url_prefix.to_owned(),
        manifest_path: source.manifest_path.to_owned(),
        signing: Arc::new(PresignedSigning::new(
            source.key,
            R2_REGION,
            RELEASE_READ_EXPIRY,
        )),
        clock,
        platform: Platform::MacosArm64,
        artifact: "macos",
    };
    match releases.current().await {
        Ok(release) => {
            tracing::info!(image_id = %release.download.image_id.0, "macOS release is on offer");
            Some(release.download)
        }
        Err(error) => {
            tracing::warn!(%error, "macOS images are off: the release read failed");
            None
        }
    }
}
