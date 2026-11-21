//! The operator reclaim command releases data and retains machine hit history.

use std::process::Command;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use ciproxy_cache_sidecar::error::CacheError;
use ciproxy_cache_sidecar::storage::disk::DiskStorage;
use ciproxy_cache_sidecar::storage::interface::{ReadSource, Storage};

#[tokio::test]
async fn offline_reclaim_removes_archive_bytes_and_preserves_report_history() {
    let directory = tempfile::tempdir().unwrap();
    let storage = DiskStorage::new(directory.path().to_path_buf(), 1024);
    storage
        .write_metrics(b"persisted machine report history")
        .await
        .unwrap();
    let (body, capture) = storage.capture(Body::from("archive"), Some(7)).await;
    assert_eq!(to_bytes(body, 1024).await.unwrap().as_ref(), b"archive");
    let archive = capture.unwrap().complete.lock().await.take().unwrap();
    assert_eq!(
        storage
            .read(&ReadSource::Capture(Arc::clone(&archive)), 0, 7)
            .await
            .unwrap()
            .as_ref(),
        b"archive"
    );
    let result = Command::new(env!("CARGO_BIN_EXE_ciproxy-cache-sidecar"))
        .args(["reclaim", "--cache-dir"])
        .arg(directory.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let restored = DiskStorage::new(directory.path().to_path_buf(), 0);
    assert_eq!(
        restored.read_metrics().await.unwrap(),
        b"persisted machine report history"
    );
    assert!(matches!(
        restored.read(&ReadSource::Capture(archive), 0, 7).await,
        Err(CacheError::Internal(_))
    ));
}
