//! Deployed runtime inputs and diagnostic image-sizing fallback.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use base64::Engine;
use base64::alphabet;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::images::ImageDownload;
use secrecy::SecretString;

use crate::error::{Error, ErrorResultExt, Result};

/// Explicit process configuration. Debug never reveals secret files.
pub struct Config {
    /// Bound HTTP socket behind the TLS ingress.
    pub http_bind: SocketAddr,
    /// PostgreSQL connection kept in the hosted process.
    pub database_url: SecretString,
    /// Externally visible dashboard origin.
    pub public_url: String,
    /// Signed-image URL prefix.
    pub image_url_prefix: String,
    /// Cloud KMS key used by the overflow composition.
    pub boat_kms_key: String,
    /// Selected CDN signing-key name.
    pub cdn_signing_key_name: String,
    /// Mounted runtime secret directory.
    pub secrets_directory: PathBuf,
    /// Public GitHub App client identity, used as the App JWT issuer.
    pub github_app_id: String,
    /// Public OAuth client identity of the same GitHub App.
    pub github_oauth_client_id: String,
    /// Registered public App slug used to derive the install URL.
    pub github_app_slug: String,
    /// Environment-specific installer origin.
    pub download_url: String,
    /// Immutable deployed commit identity.
    pub build_id: String,
    /// Current authenticated image metadata supplied by release composition.
    pub image: Option<ImageDownload>,
    /// Immutable published manifest path under IMAGE_URL_PREFIX.
    pub image_manifest_path: String,
    /// Exact released filesystem bytes from the same operator manifest.
    pub image_disk_bytes: u64,
}

impl Config {
    /// Read the chart's environment without putting secrets in diagnostics.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: env::var("HTTP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
                .parse()
                .internal()?,
            database_url: required("DATABASE_URL")?.into(),
            public_url: required("PUBLIC_URL")?,
            image_url_prefix: required("IMAGE_URL_PREFIX")?,
            boat_kms_key: required("BOAT_KMS_KEY")?,
            cdn_signing_key_name: required("CDN_SIGNING_KEY_NAME")?,
            secrets_directory: env::var_os("SECRETS_DIRECTORY")
                .map_or_else(|| PathBuf::from("/var/run/secrets/ciproxy"), PathBuf::from),
            github_app_id: required("GITHUB_APP_ID")?,
            github_oauth_client_id: required("GITHUB_OAUTH_CLIENT_ID")?,
            github_app_slug: required("GITHUB_APP_SLUG")?,
            download_url: required("DOWNLOAD_URL")?,
            build_id: required("BUILD_ID")?,
            image: None,
            image_disk_bytes: 0,
            image_manifest_path: required("JOB_IMAGE_MANIFEST_PATH")?,
        })
    }

    /// Read one chart-mounted secret as an opaque value.
    pub fn secret(&self, name: &str) -> Result<SecretString> {
        let value = std::fs::read_to_string(self.secrets_directory.join(name)).internal()?;
        Ok(value.trim_end().to_owned().into())
    }

    /// Decode the mounted 32-byte base64url dashboard key without diagnostics.
    pub fn session_key(&self) -> Result<[u8; 32]> {
        let encoded =
            std::fs::read_to_string(self.secrets_directory.join("session-key")).internal()?;
        let engine = GeneralPurpose::new(
            &alphabet::URL_SAFE,
            GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
        );
        let bytes = engine.decode(encoded.trim()).internal()?;
        bytes.as_slice().try_into().internal()
    }
}

fn required(name: &'static str) -> Result<String> {
    env::var(name).map_or_else(|_| Err(Error::MissingSetting(name)), Ok)
}

/// The image producer's manifest field, independent of installed file bytes.
#[derive(Clone, Copy, Debug)]
pub struct ImageManifest {
    /// Exact logical length of the exported ext4 root disk.
    pub root_disk_bytes: u64,
}

/// Resolve the explicit override before release metadata and the shared default.
pub fn root_sizing(override_bytes: Option<u64>, manifest: Option<ImageManifest>) -> RootSizing {
    RootSizing {
        image_root_bytes: override_bytes
            .or_else(|| manifest.map(|image| image.root_disk_bytes))
            .unwrap_or_else(|| RootSizing::default().image_root_bytes),
    }
}

#[cfg(test)]
#[path = "_tests_/config_tests.rs"]
mod config_tests;
