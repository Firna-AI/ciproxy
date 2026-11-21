//! Network download writes bytes but cannot create a verification certificate.

use async_trait::async_trait;
use ciproxy_protocol::images::ImageDownload;
use futures_util::StreamExt;
use reqwest::Client;

use crate::error::{Error, ErrorResultExt, Result};
use crate::image_artifact::PendingImage;
use crate::image_verification::DownloadDigest;

#[cfg_attr(test, unimock::unimock(api = ImageDownloadMock))]
#[async_trait]
pub(crate) trait Download: Send + Sync {
    async fn download(
        &self,
        image: &ImageDownload,
        pending: &PendingImage,
    ) -> Result<DownloadDigest>;
}

pub(crate) struct HttpsImageDownload {
    pub(crate) client: Client,
}

#[async_trait]
impl Download for HttpsImageDownload {
    async fn download(
        &self,
        image: &ImageDownload,
        pending: &PendingImage,
    ) -> Result<DownloadDigest> {
        if !image.download_url.expose_secret().starts_with("https://") {
            return Err(Error::HttpsRequired);
        }
        let mut file = pending.writer(&pending.archive())?;
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
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("receive image bytes")?;
            digest.update(&chunk);
            if digest.exceeds(image.size_bytes) {
                return Err(Error::ImageVerification);
            }
            file.write(&chunk)?;
        }
        file.sync()?;
        Ok(digest)
    }
}
