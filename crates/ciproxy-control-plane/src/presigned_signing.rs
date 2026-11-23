//! S3 presigned GET URLs (AWS Signature Version 4) for the private image store.

use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use ciproxy_protocol::secret::Secret;
use hmac::{Hmac, KeyInit, Mac};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::{Error, ErrorResultExt, Result};
use crate::image_signing::ImageSigning;

/// Bytes that Signature Version 4 sends as they are. All other bytes are escaped.
const UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

/// A read key of an S3-compatible bucket. `Debug` shows no part of it.
#[derive(Clone, Deserialize)]
pub struct StoreKey {
    access_key_id: String,
    secret_access_key: Secret,
}

impl StoreKey {
    /// Read the JSON form `{"access_key_id": "...", "secret_access_key": "..."}`.
    ///
    /// The error names only the setting, because a parser message can quote
    /// the value.
    pub fn from_json(setting: &'static str, json: &str) -> Result<Self> {
        serde_json::from_str::<Self>(json)
            .ok()
            .filter(|key| {
                !key.access_key_id.is_empty() && !key.secret_access_key.expose_secret().is_empty()
            })
            .ok_or(Error::MalformedSetting(setting))
    }
}

impl fmt::Debug for StoreKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StoreKey([REDACTED])")
    }
}

/// Signs one GET of one object, valid for a fixed period from the signing time.
pub struct PresignedSigning {
    key: StoreKey,
    region: String,
    expiry: Duration,
}

impl PresignedSigning {
    /// Sign with this key for the S3 service in this region, such as `auto` for R2.
    pub fn new(key: StoreKey, region: impl Into<String>, expiry: Duration) -> Self {
        Self {
            key,
            region: region.into(),
            expiry,
        }
    }

    fn signing_key(&self, date: &str) -> Result<Vec<u8>> {
        let secret = format!("AWS4{}", self.key.secret_access_key.expose_secret());
        let mut key = secret.into_bytes();
        for part in [date, self.region.as_str(), "s3", "aws4_request"] {
            key = hmac(&key, part)?;
        }
        Ok(key)
    }
}

impl fmt::Debug for PresignedSigning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PresignedSigning")
            .field("key", &self.key)
            .field("region", &self.region)
            .field("expiry", &self.expiry)
            .finish()
    }
}

impl ImageSigning for PresignedSigning {
    fn sign(&self, url: &str, at: DateTime<Utc>) -> Result<Secret> {
        let mut parsed = Url::parse(url).internal()?;
        let host = match (parsed.host_str(), parsed.port()) {
            (Some(host), Some(port)) => format!("{host}:{port}"),
            (Some(host), None) => host.to_owned(),
            (None, _) => return Err(Error::UnsignableUrl),
        };
        let date = at.format("%Y%m%d").to_string();
        let time = at.format("%Y%m%dT%H%M%SZ").to_string();
        let scope = format!("{date}/{}/s3/aws4_request", self.region);
        let mut query: Vec<(String, String)> = parsed
            .query_pairs()
            .map(|(name, value)| (encode(&name), encode(&value)))
            .collect();
        for (name, value) in [
            ("X-Amz-Algorithm", "AWS4-HMAC-SHA256".to_owned()),
            (
                "X-Amz-Credential",
                format!("{}/{scope}", self.key.access_key_id),
            ),
            ("X-Amz-Date", time.clone()),
            ("X-Amz-Expires", self.expiry.as_secs().to_string()),
            ("X-Amz-SignedHeaders", "host".to_owned()),
        ] {
            query.push((name.to_owned(), encode(&value)));
        }
        query.sort();
        let query = query
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("&");
        let path = parsed.path_segments().map_or_else(String::new, |segments| {
            segments
                .map(|segment| {
                    let decoded = percent_encoding::percent_decode_str(segment);
                    utf8_percent_encode(&decoded.decode_utf8_lossy(), UNRESERVED).to_string()
                })
                .collect::<Vec<_>>()
                .join("/")
        });
        let path = format!("/{path}");
        let canonical = format!("GET\n{path}\n{query}\nhost:{host}\n\nhost\nUNSIGNED-PAYLOAD");
        let digest = hex::encode(Sha256::digest(canonical.as_bytes()));
        let string_to_sign = format!("AWS4-HMAC-SHA256\n{time}\n{scope}\n{digest}");
        let signature = hex::encode(hmac(&self.signing_key(&date)?, &string_to_sign)?);
        parsed.set_path(&path);
        parsed.set_query(Some(&format!("{query}&X-Amz-Signature={signature}")));
        Ok(Secret::new(parsed.to_string()))
    }
}

fn encode(value: &str) -> String {
    utf8_percent_encode(value, UNRESERVED).to_string()
}

fn hmac(key: &[u8], message: &str) -> Result<Vec<u8>> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).internal()?;
    mac.update(message.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(test)]
#[path = "_tests_/presigned_signing_tests.rs"]
mod presigned_signing_tests;
