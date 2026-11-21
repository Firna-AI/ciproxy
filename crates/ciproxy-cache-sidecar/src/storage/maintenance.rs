//! Recovery, clear, and resize under the shared allocation lock.

use std::sync::Weak;
use std::sync::atomic::Ordering;

use tokio::fs;

use crate::error::{Result, ResultExt};

use super::disk::DiskStorage;
use super::index::{blob_names, entries};

impl DiskStorage {
    pub(super) async fn recover_files(&self) -> Result<()> {
        let mut index = self.index.lock().await;
        *index = None;
        let records = entries(&self.root).await?;
        let blobs: Vec<_> = records.iter().flat_map(blob_names).collect();
        let mut files = fs::read_dir(&self.root).await.internal()?;
        while let Some(file) = files.next_entry().await.internal()? {
            let name = file.file_name().to_string_lossy().into_owned();
            let orphan = (name.ends_with(".blob") || name.ends_with(".json"))
                && !records
                    .iter()
                    .any(|entry| blobs.contains(&name) || name == format!("{}.json", entry.id));
            if name.starts_with(".tmp") || name.ends_with(".pending") || orphan {
                fs::remove_file(file.path()).await.internal()?;
            }
        }
        self.used.store(
            records.iter().map(|entry| entry.size).sum(),
            Ordering::SeqCst,
        );
        self.allocation.reclaim(&mut index, 0).await
    }

    async fn cancel_captures(&self) -> Result<()> {
        let mut captures = self.captures.lock().await;
        for capture in captures.iter().filter_map(Weak::upgrade) {
            let _writing = capture.writes.lock().await;
            capture.release(false);
            match fs::remove_file(&capture.path).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                result => result.internal()?,
            }
        }
        captures.clear();
        Ok(())
    }

    pub(super) async fn clear_files(&self) -> Result<()> {
        let mut index = self.index.lock().await;
        let _reading = self.reads.write().await;
        *index = None;
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.cancel_captures().await?;
        fs::create_dir_all(&self.root).await.internal()?;
        let mut files = fs::read_dir(&self.root).await.internal()?;
        while let Some(file) = files.next_entry().await.internal()? {
            let name = file.file_name().to_string_lossy().into_owned();
            if name.ends_with(".json")
                || name.ends_with(".blob")
                || name.starts_with(".tmp")
                || name.ends_with(".pending") && name != "reports.pending"
            {
                fs::remove_file(file.path()).await.internal()?;
            }
        }
        self.used.store(0, Ordering::SeqCst);
        *index = Some(Vec::new());
        Ok(())
    }

    pub(super) async fn resize_files(&self, bytes: u64) -> Result<()> {
        let mut index = self.index.lock().await;
        let _reading = self.reads.write().await;
        let previous = self.capacity.swap(bytes, Ordering::SeqCst);
        self.budget_known.store(true, Ordering::SeqCst);
        if bytes < previous {
            self.generation.fetch_add(1, Ordering::SeqCst);
            self.cancel_captures().await?;
        }
        self.allocation.reclaim(&mut index, 0).await
    }
}
