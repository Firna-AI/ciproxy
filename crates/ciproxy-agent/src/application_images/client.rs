//! HTTP client policy shared by image composition and download regressions.

use std::time::Duration;

use reqwest::{Client, ClientBuilder};

pub(crate) fn builder(connect_timeout: Duration, read_timeout: Duration) -> ClientBuilder {
    Client::builder()
        .https_only(true)
        .http1_only()
        .connect_timeout(connect_timeout)
        .read_timeout(read_timeout)
}
