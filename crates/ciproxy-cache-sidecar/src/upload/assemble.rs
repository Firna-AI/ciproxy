//! Stream committed chunks in protocol order into one bounded archive.

use std::sync::Arc;

use crate::machine::MachineCache;
use crate::storage::capture::Captured;

use super::Upload;

impl MachineCache {
    pub(super) async fn assemble(&self, upload: &Upload) -> Option<Arc<Captured>> {
        if let Some(blob) = &upload.whole {
            return Some(blob.clone());
        }
        let mut chunks = Vec::new();
        if !upload.order.is_empty() {
            for id in &upload.order {
                chunks.push(upload.blocks.get(id)?.clone());
            }
        } else {
            let mut offset = 0;
            for (start, blob) in &upload.ranges {
                if *start != offset {
                    return None;
                }
                offset = offset.checked_add(blob.size)?;
                chunks.push(blob.clone());
            }
        }
        if chunks.is_empty() {
            return None;
        }
        self.store.combine(chunks).await.ok()
    }
}
