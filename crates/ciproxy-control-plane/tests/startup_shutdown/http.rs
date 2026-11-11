//! HTTP events prove serving readiness, body extraction, and request completion.

use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Duration;

use axum::http::StatusCode;
use reqwest::Client;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::task::yield_now;
use tokio::time::timeout;

/// Poll the documented readiness route until the first reconciliation succeeds.
pub(crate) async fn wait_until_ready(address: SocketAddr) -> Option<StatusCode> {
    let client = Client::builder().no_proxy().build().unwrap();
    timeout(Duration::from_secs(15), async {
        loop {
            let response = client
                .get(format!("http://{address}/api/health/ready"))
                .send()
                .await
                .expect("the child listener must answer its readiness route");
            if response.status() == StatusCode::OK {
                return response.status();
            }
            yield_now().await;
        }
    })
    .await
    .ok()
}

/// Poll new connections while the test withholds the active request's last byte.
///
/// A connection that races listener closure can reset before refusal begins.
pub(crate) async fn wait_until_refused(address: SocketAddr) -> Option<ErrorKind> {
    timeout(Duration::from_secs(10), async {
        loop {
            if let Err(error) = TcpStream::connect(address).await
                && error.kind() != ErrorKind::ConnectionReset
            {
                return error.kind();
            }
            yield_now().await;
        }
    })
    .await
    .ok()
}

/// One incomplete body keeps a real HTTP request active across SIGTERM.
pub(crate) struct PendingWebhook(BufReader<TcpStream>);

impl PendingWebhook {
    /// Request a body-read acknowledgement before sending the partial JSON body.
    pub(crate) async fn connect(address: SocketAddr) -> Self {
        let mut stream = TcpStream::connect(address).await.unwrap();
        stream
            .write_all(
                format!(
                    "POST /hooks/github HTTP/1.1\r\nHost: {address}\r\nContent-Length: 2\r\nExpect: 100-continue\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        Self(BufReader::new(stream))
    }

    /// Keep the second byte under test control after acknowledging body reads.
    pub(crate) async fn send_partial_body(&mut self) {
        self.0.get_mut().write_all(b"{").await.unwrap();
    }

    /// Finish the request and preserve an exact response or transport error.
    pub(crate) async fn complete(&mut self) -> Result<Option<StatusCode>, ErrorKind> {
        self.0
            .get_mut()
            .write_all(b"}")
            .await
            .map_err(|error| error.kind())?;
        self.read_status().await
    }

    /// Read one HTTP response head, including the informational acknowledgement.
    pub(crate) async fn read_status(&mut self) -> Result<Option<StatusCode>, ErrorKind> {
        timeout(Duration::from_secs(15), self.response_head())
            .await
            .map_err(|_| ErrorKind::TimedOut)?
    }

    async fn response_head(&mut self) -> Result<Option<StatusCode>, ErrorKind> {
        let mut line = String::new();
        let count = self
            .0
            .read_line(&mut line)
            .await
            .map_err(|error| error.kind())?;
        if count == 0 {
            return Ok(None);
        }
        let status = line
            .split_whitespace()
            .nth(1)
            .ok_or(ErrorKind::InvalidData)?;
        let status =
            StatusCode::from_bytes(status.as_bytes()).map_err(|_| ErrorKind::InvalidData)?;
        loop {
            line.clear();
            let count = self
                .0
                .read_line(&mut line)
                .await
                .map_err(|error| error.kind())?;
            if count == 0 {
                return Ok(None);
            }
            if line == "\r\n" {
                return Ok(Some(status));
            }
        }
    }
}
