//! Stream committed chunks in protocol order into one bounded archive.

use std::sync::Arc;

use crate::machine::MachineCache;
use crate::storage::capture::Captured;

use super::state::{Archive, Upload};

impl MachineCache {
    pub(super) async fn assemble(&self, upload: &Upload) -> Option<Arc<Captured>> {
        let chunks = match &upload.active {
            Archive::Whole(blob) => return blob.clone(),
            Archive::Blocks(blocks) => blocks
                .iter()
                .map(|block| block.capture.clone())
                .collect::<Option<Vec<_>>>()?,
            Archive::Ranges(ranges) => {
                let mut offset = 0;
                let mut chunks = Vec::new();
                for (start, blob) in ranges {
                    if *start != offset {
                        return None;
                    }
                    offset = offset.checked_add(blob.size)?;
                    chunks.push(blob.clone());
                }
                chunks
            }
        };
        if chunks.is_empty() {
            return None;
        }
        self.store.combine(chunks).await.ok()
    }
}
