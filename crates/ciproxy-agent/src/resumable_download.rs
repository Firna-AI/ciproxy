//! A download of one large image archive that continues from the bytes on
//! disk.
//!
//! The bytes go to `NAME.tar.partial`. The sidecar `NAME.tar.partial.json`
//! names the image ID, the SHA-256, and the byte size that the bytes belong
//! to. A later command for the same image continues the partial file with a
//! `Range` request. Inside one command, a broken stream retries a few times.
//! A `5xx` or a `429` retries in the same way, and a `429` waits as its
//! `Retry-After` header asks, up to a limit. A `Range` answer that cannot
//! continue the file empties it, and the retry asks for the whole archive.
//! Another HTTP error status ends the command and keeps the partial file,
//! because the next command brings a fresh URL. After the size and the SHA-256 match,
//! the partial file becomes `NAME.tar`.
//!
//! The macOS image store uses this module. It builds on every platform for
//! its tests.
#![cfg_attr(
    not(target_os = "macos"),
    allow(dead_code, reason = "Only the macOS image store calls it.")
)]

mod answer;
mod files;
mod partial;

use std::path::Path;
use std::time::Duration;

use ciproxy_protocol::images::ImageDownload;
use futures_util::StreamExt;
use reqwest::Client;
use reqwest::header::RANGE;

use self::answer::{Answer, read_answer};
use self::partial::Partial;
use crate::error::{Error, Result};

pub(crate) use self::files::recover;

/// How often and how fast a broken stream, a `5xx`, or a `429` retries
/// inside one command.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Retry {
    /// Retries after consecutive broken streams with no new bytes.
    pub(crate) retries: u32,
    /// The wait before each retry.
    pub(crate) wait: Duration,
    /// The longest wait that a `Retry-After` header of a `429` can ask for.
    pub(crate) longest_wait: Duration,
}

impl Retry {
    /// A few quick retries. A URL stays valid for 2 hours, so a longer
    /// outage ends the command, and the next command continues the file.
    pub(crate) const BROKEN_STREAM: Self = Self {
        retries: 3,
        wait: Duration::from_secs(5),
        longest_wait: Duration::from_secs(60),
    };
}

/// Downloads image archives that continue across breaks and commands.
pub(crate) struct ResumableDownload {
    client: Client,
    retry: Retry,
}

/// How one HTTP request ended when it did not fail the command.
enum Transfer {
    /// The body ended without an error.
    Ended,
    /// The connection or the body broke, or the host was busy. A retry can
    /// continue, after the wait that the host asked for, if any.
    Broken(Error, Option<Duration>),
}

impl ResumableDownload {
    /// Use `client` for each request and `retry` for broken streams.
    pub(crate) fn new(client: Client, retry: Retry) -> Self {
        Self { client, retry }
    }

    /// Download `image` to `archive`, `images/NAME.tar`, and check its byte
    /// count and SHA-256. Continue a partial file of the same image, and
    /// remove the partial files of other images.
    pub(crate) async fn fetch(&self, image: &ImageDownload, archive: &Path) -> Result<()> {
        let mut partial = Partial::open(image, archive).await?;
        let filled = self.fill(image, &mut partial).await;
        let flushed = partial.flush().await;
        filled?;
        flushed?;
        partial.finish(image, archive).await
    }

    /// Request the missing bytes until the partial file has `size_bytes`.
    /// Retry broken streams a few times in a row. A request that adds bytes
    /// starts the count again.
    async fn fill(&self, image: &ImageDownload, partial: &mut Partial) -> Result<()> {
        let mut failures = 0_u32;
        while partial.len() < image.size_bytes {
            let before = partial.len();
            let (error, asked) = match self.request(image, partial).await? {
                Transfer::Broken(error, asked) => (error, asked),
                Transfer::Ended if partial.len() < image.size_bytes => (Error::ImageAnswer, None),
                Transfer::Ended => break,
            };
            if partial.len() > before {
                failures = 0;
            }
            failures += 1;
            if failures > self.retry.retries {
                return Err(error);
            }
            let wait = asked.map_or(self.retry.wait, |asked| asked.min(self.retry.longest_wait));
            tracing::warn!(%error, bytes = partial.len(), wait_seconds = wait.as_secs(), "the image download continues");
            tokio::time::sleep(wait).await;
        }
        Ok(())
    }

    /// Send one request from the end of the partial file and append its body.
    async fn request(&self, image: &ImageDownload, partial: &mut Partial) -> Result<Transfer> {
        let offset = partial.len();
        let mut request = self.client.get(image.download_url.expose_secret());
        if offset > 0 {
            request = request.header(RANGE, format!("bytes={offset}-"));
        }
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => return Ok(broken(error, "send the image request")),
        };
        match read_answer(&response, offset)? {
            Answer::Whole => partial.restart().await?,
            Answer::Rest => {}
            Answer::Unusable => {
                partial.restart().await?;
                return Ok(Transfer::Broken(Error::ImageAnswer, None));
            }
            Answer::Busy {
                status,
                retry_after,
            } => return Ok(Transfer::Broken(Error::ImageStatus { status }, retry_after)),
        }
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => partial.append(&chunk, image.size_bytes).await?,
                Err(error) => return Ok(broken(error, "receive image bytes")),
            }
        }
        Ok(Transfer::Ended)
    }
}

/// A transport failure that a retry can continue. The error drops the URL,
/// because the URL holds the signature.
fn broken(error: reqwest::Error, action: &'static str) -> Transfer {
    Transfer::Broken(Error::internal_with(error.without_url(), action), None)
}

#[cfg(test)]
#[path = "_tests_/resumable_download_tests.rs"]
mod resumable_download_tests;
