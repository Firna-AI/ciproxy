//! An agent-owned temporary download directory, never an external archive path.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::budget_file::BudgetFile;
use crate::error::{Error, Result};
use crate::image_space::ImageSpace;

/// A temporary directory allocated by the image store.
#[derive(Clone)]
pub struct PendingImage {
    directory: PathBuf,
    space: Option<Arc<dyn ImageSpace>>,
}

impl PendingImage {
    #[cfg(test)]
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            space: None,
        }
    }

    pub(crate) fn bounded(directory: PathBuf, space: Arc<dyn ImageSpace>) -> Self {
        Self {
            directory,
            space: Some(space),
        }
    }

    pub(crate) fn writer(&self, path: &Path) -> Result<BudgetFile> {
        BudgetFile::create(path, self.space.clone().ok_or(Error::ImageOwnership)?)
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn archive(&self) -> PathBuf {
        self.directory.join("image.tar.zst")
    }
}

impl PartialEq for PendingImage {
    fn eq(&self, other: &Self) -> bool {
        self.directory == other.directory
    }
}
impl Eq for PendingImage {}
impl fmt::Debug for PendingImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PendingImage")
            .field("directory", &self.directory)
            .finish()
    }
}
