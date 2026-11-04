//! Streaming HTTP transport with explicit destinations and no credential redirects.

use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Response};
use reqwest::{Client, redirect};

use crate::error::{Result, ResultExt};

/// A streaming request to a destination chosen by trusted configuration.
pub struct Outbound {
    /// HTTP method.
    pub method: Method,
    /// Upstream URL. Never take this from job JSON or a job header.
    pub url: String,
    /// End-to-end request headers.
    pub headers: HeaderMap,
    /// Streaming request contents.
    pub body: Body,
}

/// Swappable streaming HTTP boundary.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Forward one request, retaining the upstream status and headers.
    async fn send(&self, request: Outbound) -> Result<Response<Body>>;
}

/// The production HTTP transport.
pub struct HttpTransport {
    client: Client,
}

impl HttpTransport {
    /// Construct a transport with bounded connection and transfer time.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(600))
                .build()
                .internal()?,
        })
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&self, request: Outbound) -> Result<Response<Body>> {
        let response = self
            .client
            .request(request.method, &request.url)
            .headers(clean_headers(request.headers))
            .body(reqwest::Body::wrap_stream(request.body.into_data_stream()))
            .send()
            .await
            .internal()?;
        let status = response.status();
        let headers = clean_headers(response.headers().clone());
        let mut result = Response::new(Body::from_stream(response.bytes_stream()));
        *result.status_mut() = status;
        *result.headers_mut() = headers;
        Ok(result)
    }
}

pub(crate) fn clean_headers(mut headers: HeaderMap) -> HeaderMap {
    if let Some(value) = headers
        .get("connection")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
    {
        for name in value.split(',') {
            headers.remove(name.trim());
        }
    }
    for name in [
        "host",
        "connection",
        "transfer-encoding",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "upgrade",
    ] {
        headers.remove(name);
    }
    headers
}
