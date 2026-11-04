//! One replacement rule for legacy ranges and signed blob upload pieces.

use std::sync::Arc;

use crate::storage::capture::Captured;

use super::Upload;

#[derive(Clone)]
pub(super) enum UploadPart {
    Whole,
    Block(String),
    Range { start: u64, end: u64 },
    UnspecifiedRange,
}

impl Upload {
    /// Remove previous bytes before capture so any failed replacement leaves a hole.
    pub(super) fn replace(&mut self, part: UploadPart, blob: Option<Arc<Captured>>) {
        match part {
            UploadPart::Whole => self.whole = blob,
            UploadPart::Block(id) => {
                self.blocks.remove(&id);
                if let Some(blob) = blob {
                    self.blocks.insert(id, blob);
                }
            }
            UploadPart::Range { start, end } => {
                self.ranges.remove(&start);
                if let Some(blob) = blob
                    && Some(blob.size)
                        == end.checked_sub(start).and_then(|size| size.checked_add(1))
                {
                    self.ranges.insert(start, blob);
                }
            }
            UploadPart::UnspecifiedRange => self.ranges.clear(),
        }
    }
}
