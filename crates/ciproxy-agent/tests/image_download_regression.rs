//! Real HTTP bodies exercise the production image client's read timeout policy.

#[path = "../src/application_images/client.rs"]
mod client;

use std::task::Poll;
use std::time::Duration;

use futures_util::poll;
use reqwest::Response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{advance, pause, resume};

const READ_TIMEOUT: Duration = Duration::from_secs(30);
/// Tokio rounds timer deadlines up to the next millisecond.
const TIMER_TICK: Duration = Duration::from_millis(1);

struct ImageServer {
    url: String,
    bytes: mpsc::Sender<u8>,
    task: JoinHandle<()>,
}

impl ImageServer {
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/image", listener.local_addr().unwrap());
        let (bytes, mut receive) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\n")
                .await
                .unwrap();
            while let Some(byte) = receive.recv().await {
                socket.write_all(&[byte]).await.unwrap();
            }
        });
        Self { url, bytes, task }
    }

    /// Keep real socket IO on a live clock, then freeze before body reads start.
    async fn response(&self) -> Response {
        let client = client::builder(Duration::from_secs(20), READ_TIMEOUT)
            .https_only(false)
            .no_proxy()
            .build()
            .unwrap();
        resume();
        let response = client.get(&self.url).send().await.unwrap();
        pause();
        response
    }
}

impl Drop for ImageServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tokio::test(start_paused = true)]
async fn completes_body_that_keeps_receiving_bytes_past_total_limit() {
    let server = ImageServer::start().await;
    let mut response = server.response().await;
    let mut body = Vec::new();
    for byte in b"slow" {
        let chunk = response.chunk();
        tokio::pin!(chunk);
        assert!(poll!(chunk.as_mut()).is_pending());
        advance(Duration::from_secs(10)).await;
        assert!(poll!(chunk.as_mut()).is_pending());
        resume();
        server.bytes.send(*byte).await.unwrap();
        body.extend_from_slice(&chunk.await.unwrap().unwrap());
        pause();
    }
    assert_eq!(body, b"slow");
    assert_eq!(response.chunk().await.unwrap(), None);
}

#[tokio::test(start_paused = true)]
async fn fails_body_that_stalls_at_read_limit() {
    let server = ImageServer::start().await;
    let mut response = server.response().await;
    resume();
    server.bytes.send(b's').await.unwrap();
    assert_eq!(response.chunk().await.unwrap().unwrap().as_ref(), b"s");
    pause();

    let chunk = response.chunk();
    tokio::pin!(chunk);
    assert!(poll!(chunk.as_mut()).is_pending());
    advance(READ_TIMEOUT - Duration::from_secs(1)).await;
    assert!(poll!(chunk.as_mut()).is_pending());
    advance(Duration::from_secs(1) + TIMER_TICK).await;
    let Poll::Ready(Err(error)) = poll!(chunk.as_mut()) else {
        panic!("a stalled body must fail at the read limit");
    };
    assert!(error.is_timeout(), "expected a read timeout: {error}");
}
