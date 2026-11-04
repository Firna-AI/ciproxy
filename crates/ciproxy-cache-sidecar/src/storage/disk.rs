//! Persistent files with atomic publication, one byte budget, and cancellation.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use axum::body::Body;
use bytes::Bytes;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{CacheError, Result, ResultExt};

use super::capture::{self, Capture, Captured, Reservation, hex};
use super::index::{blob_names, entries, matching, next_order, remove};
use super::interface::{Entry, Lookup, ReadSource, Storage};
use super::publication::persist;
use super::{assembly, read};

/// Filesystem storage. A machine must have only one service for this root.
pub struct DiskStorage {
    root: PathBuf,
    capacity: Arc<AtomicU64>,
    used: Arc<AtomicU64>,
    generation: AtomicU64,
    captures: Mutex<Vec<Weak<Reservation>>>,
    lock: Mutex<()>,
}

impl DiskStorage {
    /// Select a dedicated cache directory and its byte budget.
    pub fn new(root: PathBuf, capacity: u64) -> Self {
        Self {
            root,
            capacity: Arc::new(AtomicU64::new(capacity)),
            used: Arc::new(AtomicU64::new(0)),
            generation: AtomicU64::new(0),
            captures: Mutex::new(Vec::new()),
            lock: Mutex::new(()),
        }
    }

    async fn invalidate(&self, entry: &Entry) -> Result<()> {
        let _guard = self.lock.lock().await;
        if entries(&self.root)
            .await?
            .iter()
            .any(|candidate| candidate.id == entry.id)
        {
            remove(&self.root, entry).await?;
            self.used.fetch_sub(entry.size, Ordering::SeqCst);
        }
        Ok(())
    }

    async fn cancel_captures(&self) -> Result<()> {
        let mut captures = self.captures.lock().await;
        for capture in captures.iter().filter_map(Weak::upgrade) {
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

    async fn free_for(&self, bytes: u64) -> Result<()> {
        let mut records = entries(&self.root).await?;
        records.sort_by_key(|entry| (entry.accessed, entry.created, entry.id.clone()));
        for entry in records {
            if self.used.load(Ordering::SeqCst).saturating_add(bytes)
                <= self.capacity.load(Ordering::SeqCst)
            {
                break;
            }
            remove(&self.root, &entry).await?;
            self.used.fetch_sub(entry.size, Ordering::SeqCst);
        }
        Ok(())
    }
}

#[async_trait]
impl Storage for DiskStorage {
    async fn recover(&self) -> Result<()> {
        let _guard = self.lock.lock().await;
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
        self.free_for(0).await
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    async fn read(&self, source: &ReadSource, offset: u64, length: usize) -> Result<Bytes> {
        read::read(&self.root, source, offset, length).await
    }

    async fn lookup(&self, lookup: &Lookup) -> Result<Option<Entry>> {
        let _guard = self.lock.lock().await;
        Ok(matching(&entries(&self.root).await?, lookup))
    }

    async fn verify(&self, entry: &Entry) -> Result<()> {
        let mut digest = Sha256::new();
        let mut offset = 0;
        loop {
            let bytes = match self
                .read(&ReadSource::Entry(Arc::new(entry.clone())), offset, 65536)
                .await
            {
                Ok(bytes) => bytes,
                Err(error) => {
                    self.invalidate(entry).await?;
                    return Err(error);
                }
            };
            if bytes.is_empty() {
                break;
            }
            offset += bytes.len() as u64;
            digest.update(&bytes);
        }
        if offset != entry.size || hex(&digest.finalize()) != entry.digest {
            self.invalidate(entry).await?;
            return Err(CacheError::Unavailable);
        }
        Ok(())
    }

    async fn touch(&self, entry: &Entry) -> Result<()> {
        let _guard = self.lock.lock().await;
        let path = self.root.join(format!("{}.json", entry.id));
        if fs::metadata(&path).await.is_ok() {
            let mut entry = entry.clone();
            entry.accessed = next_order(
                entries(&self.root)
                    .await?
                    .iter()
                    .map(|entry| entry.accessed),
            )?;
            let temporary = self.root.join(format!("{}.pending", entry.id));
            fs::write(&temporary, serde_json::to_vec(&entry).internal()?)
                .await
                .internal()?;
            fs::rename(temporary, path).await.internal()?;
        }
        Ok(())
    }

    async fn capture(&self, body: Body, expected: Option<u64>) -> (Body, Option<Arc<Capture>>) {
        let _guard = self.lock.lock().await;
        let required = expected.unwrap_or(self.capacity.load(Ordering::SeqCst));
        if required > self.capacity.load(Ordering::SeqCst) || self.free_for(required).await.is_err()
        {
            return (body, None);
        }
        match NamedTempFile::new_in(&self.root) {
            Ok(file) => {
                let (body, capture) = capture::tee(
                    file,
                    body,
                    self.used.clone(),
                    self.capacity.clone(),
                    self.generation(),
                    expected,
                );
                let mut captures = self.captures.lock().await;
                captures.retain(|capture| capture.strong_count() > 0);
                captures.push(Arc::downgrade(&capture.reservation));
                (body, Some(capture))
            }
            Err(_) => (body, None),
        }
    }

    async fn combine(&self, chunks: Vec<Arc<Captured>>) -> Result<Arc<Captured>> {
        let _guard = self.lock.lock().await;
        assembly::combine(
            &self.root,
            self,
            chunks,
            self.generation(),
            self.used.clone(),
            self.capacity.clone(),
        )
        .await
    }

    async fn publish(&self, mut entry: Entry, blob: &Captured) -> Result<Entry> {
        let _guard = self.lock.lock().await;
        if blob.generation != self.generation() {
            return Err(CacheError::Unavailable);
        }
        let records = entries(&self.root).await?;
        if let Some(old) = records.iter().find(|old| {
            old.scope == entry.scope && old.key == entry.key && old.version == entry.version
        }) {
            return Ok(old.clone());
        }
        entry.created = next_order(records.iter().map(|entry| entry.created))?;
        entry.accessed = next_order(records.iter().map(|entry| entry.accessed))?;
        entry.parts = blob.parts.iter().map(|part| part.size).collect();
        entry.size = blob.size;
        entry.digest = blob.digest.clone();
        entry.id = Uuid::new_v4().to_string();
        persist(&self.root, &entry, blob).await?;
        Ok(entry)
    }

    async fn clear(&self) -> Result<()> {
        let _guard = self.lock.lock().await;
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
        Ok(())
    }

    async fn resize(&self, bytes: u64) -> Result<()> {
        let _guard = self.lock.lock().await;
        let previous = self.capacity.swap(bytes, Ordering::SeqCst);
        if bytes < previous {
            self.generation.fetch_add(1, Ordering::SeqCst);
            self.cancel_captures().await?;
        }
        self.free_for(0).await
    }

    async fn usage(&self) -> Result<(u64, u64)> {
        Ok((
            self.used.load(Ordering::SeqCst),
            self.capacity.load(Ordering::SeqCst),
        ))
    }

    async fn read_metrics(&self) -> Result<Vec<u8>> {
        match fs::read(self.root.join("reports")).await {
            Ok(bytes) => Ok(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            result => result.internal(),
        }
    }

    async fn write_metrics(&self, bytes: &[u8]) -> Result<()> {
        fs::create_dir_all(&self.root).await.internal()?;
        fs::write(self.root.join("reports.pending"), bytes)
            .await
            .internal()?;
        fs::rename(self.root.join("reports.pending"), self.root.join("reports"))
            .await
            .internal()
    }
}
