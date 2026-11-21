//! Ordered block archives with no duplicate archive on disk.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::error::{CacheError, Result, ResultExt};

use super::capture::{Captured, Reservation, hex};
use super::interface::{ReadSource, Storage};

pub(super) async fn combine(
    root: &Path,
    store: &dyn Storage,
    chunks: Vec<Arc<Captured>>,
    generation: u64,
    used: Arc<AtomicU64>,
    capacity: Arc<AtomicU64>,
) -> Result<Arc<Captured>> {
    let mut digest = Sha256::new();
    let mut size = 0_u64;
    for chunk in &chunks {
        if chunk.generation != generation || !chunk.parts.is_empty() {
            return Err(CacheError::Unavailable);
        }
        let mut offset = 0;
        while offset < chunk.size {
            let bytes = store
                .read(&ReadSource::Capture(chunk.clone()), offset, 65536)
                .await?;
            if bytes.is_empty() {
                return Err(CacheError::Unavailable);
            }
            offset += bytes.len() as u64;
            digest.update(&bytes);
        }
        size = size.checked_add(offset).ok_or(CacheError::Budget)?;
    }
    let path = NamedTempFile::new_in(root).internal()?.into_temp_path();
    let reservation = Arc::new(Reservation::new(path.to_path_buf(), used, capacity));
    Ok(Arc::new(Captured {
        path,
        size,
        digest: hex(&digest.finalize()),
        generation,
        reservation,
        parts: chunks,
    }))
}
