//! One replacement rule for legacy ranges and signed blob upload pieces.

use std::sync::Arc;

use serde::Deserialize;

use crate::storage::capture::Captured;

use super::state::{Archive, CommittedBlock, Upload};

#[derive(Clone, Deserialize)]
pub(super) enum BlockSelection {
    Latest(String),
    Uncommitted(String),
    Committed(String),
}

#[derive(Clone)]
pub(super) enum UploadPart {
    Whole,
    Block(String),
    Range { start: u64, end: u64 },
    UnspecifiedRange,
    Commit(Vec<BlockSelection>),
}

impl Upload {
    /// Own staged replacements and successful transitions of the active archive.
    pub(super) fn replace(&mut self, part: UploadPart, blob: Option<Arc<Captured>>) {
        match part {
            UploadPart::Whole => {
                self.active = Archive::Whole(blob);
                self.staged.clear();
            }
            UploadPart::Block(id) => {
                self.staged.insert(id, blob);
            }
            UploadPart::Range { start, end } => {
                let Archive::Ranges(ranges) = &mut self.active else {
                    return;
                };
                ranges.remove(&start);
                if let Some(blob) = blob
                    && Some(blob.size)
                        == end.checked_sub(start).and_then(|size| size.checked_add(1))
                {
                    ranges.insert(start, blob);
                }
            }
            UploadPart::UnspecifiedRange => {
                if let Archive::Ranges(ranges) = &mut self.active {
                    ranges.clear();
                }
            }
            UploadPart::Commit(selection) => {
                let blocks = selection
                    .into_iter()
                    .map(|block| {
                        let (id, capture) = match block {
                            BlockSelection::Committed(id) => {
                                let capture = self.committed(&id);
                                (id, capture)
                            }
                            BlockSelection::Uncommitted(id) => {
                                let capture = self.staged.get(&id).cloned().flatten();
                                (id, capture)
                            }
                            BlockSelection::Latest(id) => {
                                let capture = match self.staged.get(&id) {
                                    Some(capture) => capture.clone(),
                                    None => self.committed(&id),
                                };
                                (id, capture)
                            }
                        };
                        CommittedBlock { id, capture }
                    })
                    .collect();
                self.active = Archive::Blocks(blocks);
                self.staged.clear();
            }
        }
    }

    fn committed(&self, id: &str) -> Option<Arc<Captured>> {
        match &self.active {
            Archive::Blocks(blocks) => blocks
                .iter()
                .find(|block| block.id == id)
                .and_then(|block| block.capture.clone()),
            _ => None,
        }
    }
}
