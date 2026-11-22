//! A local release origin accepts the real client's signed discovery URLs.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

/// Notify the test when discovery starts and keep that response pending.
pub(super) fn pending() -> (Router, Arc<Notify>) {
    let entered = Arc::new(Notify::new());
    let observed = entered.clone();
    let routes = Router::new().route(
        "/images/release/manifest.json",
        get(move || {
            let observed = observed.clone();
            async move {
                observed.notify_one();
                std::future::pending::<(StatusCode, &'static str)>().await
            }
        }),
    );
    (routes, entered)
}

/// Supply a pinned manifest and the matching archive metadata for real startup.
pub(super) fn released() -> Router {
    Router::new()
        .route(
            "/images/release/manifest.json",
            get(|| async {
                (
                    StatusCode::OK,
                    r#"{"image_id":"shutdown-release","root_disk_bytes":62008590336}"#,
                )
            }),
        )
        .route(
            "/images/release/downloads.json",
            get(|| async {
                (
                    StatusCode::OK,
                    r#"{"firecracker":{"file":"firecracker.tar.zst","sha256":"fixture-checksum","size_bytes":12345}}"#,
                )
            }),
        )
}

/// The Linux release, and a macOS store that refuses each read and records it.
pub(super) fn released_with_forbidden_macos() -> (Router, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = requests.clone();
    let routes = released().route(
        "/ciproxy-macos-images/releases/macos-startup/manifest.json",
        get(move |OriginalUri(uri): OriginalUri| {
            recorded.lock().unwrap().push(uri.to_string());
            async { StatusCode::FORBIDDEN }
        }),
    );
    (routes, requests)
}

/// Keep the origin bound to its allocated port until fixture cleanup.
pub(super) async fn serve(routes: Router) -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, routes).await.unwrap();
    });
    (address, server)
}
