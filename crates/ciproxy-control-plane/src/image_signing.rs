//! Image URL signers: Cloud CDN for Linux, and the signer of each platform.

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use chrono::{DateTime, Utc};
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::secret::Secret;
use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::ImageOffer;

/// Delivery-time image URL signing, independent from release metadata.
pub trait ImageSigning: Send + Sync {
    /// Sign the configured artifact for this delivery's bounded download period.
    fn sign(&self, url: &str, at: DateTime<Utc>) -> Result<Secret>;
}

/// The selected CDN key stays inside this hosted signer.
pub struct CdnSigning {
    key_name: String,
    key: Vec<u8>,
}

impl CdnSigning {
    /// Decode the mounted key registered with the existing CDN backend.
    pub fn new(key_name: String, encoded: &str) -> Result<Self> {
        Ok(Self {
            key_name,
            key: URL_SAFE.decode(encoded.trim()).internal()?,
        })
    }
}

impl ImageSigning for CdnSigning {
    fn sign(&self, url: &str, at: DateTime<Utc>) -> Result<Secret> {
        let separator = if url.contains('?') { '&' } else { '?' };
        let expires = (at + chrono::Duration::hours(2)).timestamp();
        let input = format!(
            "{url}{separator}Expires={expires}&KeyName={}",
            self.key_name
        );
        let mut mac = Hmac::<Sha1>::new_from_slice(&self.key).internal()?;
        mac.update(input.as_bytes());
        let signature = URL_SAFE.encode(mac.finalize().into_bytes());
        Ok(Secret::new(format!("{input}&Signature={signature}")))
    }
}

/// The URL signer of the release of each platform.
pub struct PlatformSigning {
    /// Signs Linux archive URLs with the CDN key.
    pub linux: Arc<dyn ImageSigning>,
    /// Signs macOS archive URLs for the private R2 store, while macOS images are on.
    pub macos: Option<Arc<dyn ImageSigning>>,
}

impl PlatformSigning {
    /// Offer each release with the signer of its platform.
    pub fn offers(&self, images: &[ImageDownload]) -> Vec<ImageOffer> {
        images
            .iter()
            .map(|download| ImageOffer {
                signing: match download.platform {
                    Platform::LinuxX64 => Some(self.linux.clone()),
                    Platform::MacosArm64 => self.macos.clone(),
                },
                download: download.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "_tests_/image_signing_tests.rs"]
mod image_signing_tests;
