//! Cloud CDN signed image URLs issued at each agent delivery.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use chrono::{DateTime, Utc};
use ciproxy_protocol::secret::Secret;
use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

use crate::error::{ErrorResultExt, Result};

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

#[cfg(test)]
#[path = "_tests_/image_signing_tests.rs"]
mod image_signing_tests;
