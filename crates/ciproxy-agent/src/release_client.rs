//! Shared HTTPS-only metadata and archive transport for both host executables.

use std::time::Duration;

use async_trait::async_trait;
use cli_release_interface::ReleaseAssetInfo;
use cli_updater::{Error, ReleaseClient, Result};
use reqwest::{Client, Response, Url, redirect};

pub(crate) struct SecureReleases {
    base: String,
    http: Client,
}

impl SecureReleases {
    pub(crate) fn new(base: String) -> Result<Self> {
        let http = match Client::builder()
            .https_only(true)
            .http1_only()
            .redirect(redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
        {
            Ok(http) => http,
            Err(source) => return Err(Error::ReleaseServerClientBuild { source }),
        };
        Ok(Self { base, http })
    }
    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
    async fn get(&self, url: &str) -> Result<Response> {
        let response = match self.http.get(url).send().await {
            Ok(response) => response,
            Err(source) => {
                return Err(Error::ReleaseServerRequest {
                    url: url.to_owned(),
                    source,
                });
            }
        };
        if !response.status().is_success() {
            return Err(Error::ReleaseServerStatus {
                url: url.to_owned(),
                status: response.status().as_u16(),
            });
        }
        Ok(response)
    }
}

#[async_trait]
impl ReleaseClient for SecureReleases {
    async fn latest(&self, binary: &str, target: &str) -> Result<ReleaseAssetInfo> {
        let url = self.endpoint(&format!("releases/latest/{binary}/{target}"));
        match self.get(&url).await?.json().await {
            Ok(release) => Ok(release),
            Err(source) => Err(Error::ReleaseServerDecode { url, source }),
        }
    }
    async fn download(&self, release: &ReleaseAssetInfo) -> Result<Vec<u8>> {
        let url = match Url::parse(&release.download_url) {
            Ok(url) => url.to_string(),
            Err(_) => self.endpoint(&release.download_url),
        };
        match self.get(&url).await?.bytes().await {
            Ok(bytes) => Ok(bytes.to_vec()),
            Err(source) => Err(Error::ReleaseServerDecode { url, source }),
        }
    }
}
