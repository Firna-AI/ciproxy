//! An agent-owned temporary download directory, never an external archive path.

use std::path::{Path, PathBuf};

/// A temporary directory allocated by the image store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingImage {
    directory: PathBuf,
}

impl PendingImage {
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn archive(&self) -> PathBuf {
        self.directory.join("image.tar.gz")
    }
}
