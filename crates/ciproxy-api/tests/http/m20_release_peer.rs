//! Local release metadata peer for the production HTTP release adapter.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::get};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub(super) struct Peer {
    pub(super) origin: String,
    pub(super) hits: Arc<AtomicUsize>,
    pub(super) status: Arc<AtomicU16>,
    task: JoinHandle<()>,
}

impl Peer {
    pub(super) async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let hits = Arc::new(AtomicUsize::new(0));
        let status = Arc::new(AtomicU16::new(200));
        let router = Router::new()
            .route(
                "/releases/latest/ciproxy-agent/x86_64-unknown-linux-gnu",
                get(latest),
            )
            .with_state((hits.clone(), status.clone()));
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            origin,
            hits,
            status,
            task,
        }
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn latest(State((hits, status)): State<(Arc<AtomicUsize>, Arc<AtomicU16>)>) -> Response {
    hits.fetch_add(1, Ordering::SeqCst);
    let status = StatusCode::from_u16(status.load(Ordering::SeqCst)).unwrap();
    (status, Json(json!({"binary":"ciproxy-agent", "target":"x86_64-unknown-linux-gnu", "version":"0.2.0"}))).into_response()
}
