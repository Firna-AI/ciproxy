//! Atomic archive publication with explicit rollback accounting.

use std::path::{Path, PathBuf};

use tokio::fs;

use crate::error::{Result, ResultExt};

use super::capture::Captured;
use super::index::blob_names;
use super::interface::Entry;

pub(super) async fn persist(root: &Path, entry: &Entry, blob: &Captured) -> Result<()> {
    let metadata = root.join(format!("{}.json", entry.id));
    let temporary = root.join(format!("{}.pending", entry.id));
    if let Err(error) = fs::write(&temporary, serde_json::to_vec(entry).internal()?).await {
        let _ = fs::remove_file(&temporary).await;
        return Err(error).internal();
    }
    let chunks: Vec<_> = if blob.parts.is_empty() {
        vec![blob]
    } else {
        blob.parts.iter().map(|part| part.as_ref()).collect()
    };
    let mut moved = Vec::new();
    for (chunk, name) in chunks.iter().zip(blob_names(entry)) {
        let destination = root.join(name);
        if let Err(error) = fs::rename(&chunk.path, &destination).await {
            rollback(&moved).await;
            let _ = fs::remove_file(&temporary).await;
            return Err(error).internal();
        }
        moved.push((*chunk, destination));
    }
    if let Err(error) = fs::rename(&temporary, &metadata).await {
        rollback(&moved).await;
        let _ = fs::remove_file(&temporary).await;
        return Err(error).internal();
    }
    for chunk in chunks {
        chunk.reservation.release(true);
    }
    blob.reservation.release(true);
    Ok(())
}

async fn rollback(moved: &[(&Captured, PathBuf)]) {
    for (chunk, destination) in moved {
        if fs::rename(destination, &chunk.path).await.is_err()
            && fs::remove_file(destination).await.is_err()
        {
            chunk.reservation.release(true);
            tracing::warn!("Unremovable cache orphan remains charged to the disk budget");
        }
    }
}
