//! HTTPS downloads and atomic publication of the checked official executable pair.

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::sync::Mutex;

use crate::error::{ErrorResultExt, Result};
use crate::host_release::{CheckedTools, LIMIT, Release, URL};
use crate::host_system::repair;

#[cfg_attr(test, unimock::unimock(api = HostDownloadMock))]
#[async_trait]
pub(crate) trait HostDownload: Send + Sync {
    async fn fetch(&self) -> Result<Vec<u8>>;
}

#[cfg_attr(test, unimock::unimock(api = ToolFilesMock))]
pub(crate) trait ToolFiles: Send + Sync {
    fn valid(&self) -> Result<bool>;
    fn publish(&self, checked: &CheckedTools) -> Result<()>;
    fn remove(&self) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = ToolsMock))]
#[async_trait]
pub(crate) trait Tools: Send + Sync {
    async fn prepare(&self) -> Result<()>;
    fn remove(&self) -> Result<()>;
}

pub(crate) struct VerifiedTools {
    pub(crate) download: Arc<dyn HostDownload>,
    pub(crate) files: Arc<dyn ToolFiles>,
    pub(crate) release: Release,
    pub(crate) operation: Mutex<()>,
}

#[async_trait]
impl Tools for VerifiedTools {
    async fn prepare(&self) -> Result<()> {
        let _operation = self.operation.lock().await;
        if self.files.valid()? {
            return Ok(());
        }
        let bytes = self.download.fetch().await?;
        let checked = CheckedTools::from_archive(&bytes, self.release)?;
        self.files.publish(&checked)
    }

    fn remove(&self) -> Result<()> {
        self.files.remove()
    }
}

pub(crate) struct HttpsHostDownload {
    pub(crate) client: Client,
}

#[async_trait]
impl HostDownload for HttpsHostDownload {
    async fn fetch(&self) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(URL)
            .send()
            .await
            .context(
                "download official Firecracker release; check outbound HTTPS and rerun install",
            )?
            .error_for_status()
            .context("fetch pinned Firecracker release; check GitHub access and rerun install")?;
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("read pinned Firecracker release; rerun install")?;
            if bytes.len().saturating_add(chunk.len()) as u64 > LIMIT {
                return Err(repair(
                    "Firecracker download size",
                    "Retry the pinned release download from GitHub.",
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
}

#[cfg(test)]
#[path = "_tests_/host_tools_tests.rs"]
mod host_tools_tests;
