//! Real HTTP upload streams with explicit arrival synchronization.

use std::io;

use bytes::Bytes;
use futures_util::stream;
use reqwest::{Body, Response, StatusCode};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::fixture::Fixture;
use super::job::Job;

pub fn chunked(data: &'static str) -> Body {
    Body::wrap_stream(stream::iter([Ok::<_, io::Error>(Bytes::from_static(
        data.as_bytes(),
    ))]))
}

pub async fn empty_upload(job: &Job, key: &str) -> StatusCode {
    let url = job.reserve(key, "v1").await;
    job.client
        .put(url)
        .body(Body::wrap_stream(
            stream::empty::<Result<Bytes, io::Error>>(),
        ))
        .send()
        .await
        .unwrap()
        .status()
}

pub async fn stalled_upload(fixture: &Fixture, job: &Job, key: &str, length: u64) -> TcpStream {
    let upload = url::Url::parse(&job.reserve(key, "v1").await).unwrap();
    let machine = url::Url::parse(&fixture.machine).unwrap();
    let mut connection = TcpStream::connect(machine.socket_addrs(|| None).unwrap()[0])
        .await
        .unwrap();
    connection.write_all(format!("PUT {} HTTP/1.1\r\nHost: localhost\r\nX-Ciproxy-Session: {}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n", upload.path(), job.token.expose_secret()).as_bytes()).await.unwrap();
    fixture.wait_upload(key, 0).await;
    connection
}

pub async fn streaming_upload(
    job: &Job,
    key: &str,
) -> (mpsc::Sender<Bytes>, JoinHandle<reqwest::Result<Response>>) {
    let url = job.reserve(key, "v1").await;
    let (sender, receiver) = mpsc::channel::<Bytes>(1);
    let stream = stream::unfold(receiver, |mut receiver| async move {
        receiver
            .recv()
            .await
            .map(|bytes| (Ok::<_, io::Error>(bytes), receiver))
    });
    let client = job.client.clone();
    let request =
        tokio::spawn(async move { client.put(url).body(Body::wrap_stream(stream)).send().await });
    (sender, request)
}
