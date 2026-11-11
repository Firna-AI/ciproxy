//! Persistent files with atomic publication, one byte budget, and cancellation.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use axum::body::Body;
use bytes::Bytes;
use ciproxy_protocol::cache_metadata::CacheEntryMetadata;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::error::{CacheError, Result, ResultExt};

use super::allocation::{Allocation, DiskAllocation};
use super::capture::{self, Capture, Captured, Reservation, hex};
use super::index::{entries, next_order, remove};
use super::interface::{Entry, EntryOrigin, ReadSource, Storage};
use super::publication::persist;
use super::{assembly, metadata, read};

/// Filesystem storage. A machine must have only one service for this root.
pub struct DiskStorage {
    pub(super) root: PathBuf,
    pub(super) capacity: Arc<AtomicU64>,
    pub(super) budget_known: Arc<AtomicBool>,
    pub(super) used: Arc<AtomicU64>,
    pub(super) generation: AtomicU64,
    pub(super) captures: Mutex<Vec<Weak<Reservation>>>,
    pub(super) index: Arc<Mutex<Option<Vec<Entry>>>>,
    pub(super) allocation: Arc<dyn Allocation>,
    pub(super) reads: RwLock<()>,
}

impl DiskStorage {
    /// Select a dedicated cache directory and its byte budget.
    pub fn new(root: PathBuf, capacity: u64) -> Self {
        Self::with_budget(root, Some(capacity))
    }

    /// Recover retained entries without new capture or eviction until a budget arrives.
    pub fn waiting_for_budget(root: PathBuf) -> Self {
        Self::with_budget(root, None)
    }

    fn with_budget(root: PathBuf, budget: Option<u64>) -> Self {
        let budget_known = Arc::new(AtomicBool::new(budget.is_some()));
        let capacity = budget.unwrap_or(0);
        let used = Arc::new(AtomicU64::new(0));
        let capacity = Arc::new(AtomicU64::new(capacity));
        let index = Arc::new(Mutex::new(None));
        let allocation = Arc::new(DiskAllocation::new(
            root.clone(),
            used.clone(),
            capacity.clone(),
            budget_known.clone(),
            index.clone(),
        ));
        Self {
            root,
            capacity,
            budget_known,
            used,
            generation: AtomicU64::new(0),
            captures: Mutex::new(Vec::new()),
            index,
            allocation,
            reads: RwLock::new(()),
        }
    }

    async fn invalidate(&self, entry: &Entry) -> Result<()> {
        let mut index = self.index.lock().await;
        if entries(&self.root)
            .await?
            .iter()
            .any(|candidate| candidate.id == entry.id)
        {
            remove(&self.root, entry).await?;
            self.used.fetch_sub(entry.size, Ordering::SeqCst);
            *index = None;
        }
        Ok(())
    }
}

#[async_trait]
impl Storage for DiskStorage {
    async fn recover(&self) -> Result<()> {
        self.recover_files().await
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    async fn read(&self, source: &ReadSource, offset: u64, length: usize) -> Result<Bytes> {
        let _reading = self.reads.read().await;
        read::read(&self.root, source, offset, length).await
    }

    async fn entries(&self) -> Result<Vec<Entry>> {
        let _guard = self.index.lock().await;
        entries(&self.root).await
    }

    async fn annotate_source(&self, entry: &Entry, source: &CacheEntryMetadata) -> Result<()> {
        let mut index = self.index.lock().await;
        metadata::annotate(&self.root, entry, source).await?;
        *index = None;
        Ok(())
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
        let mut index = self.index.lock().await;
        let path = self.root.join(format!("{}.json", entry.id));
        if fs::metadata(&path).await.is_ok() {
            let bytes = fs::read(&path).await.internal()?;
            let mut current: Entry = serde_json::from_slice(&bytes).internal()?;
            current.accessed = next_order(
                entries(&self.root)
                    .await?
                    .iter()
                    .map(|entry| entry.accessed),
            )?;
            let temporary = self.root.join(format!("{}.pending", entry.id));
            fs::write(&temporary, serde_json::to_vec(&current).internal()?)
                .await
                .internal()?;
            fs::rename(temporary, path).await.internal()?;
            *index = None;
        }
        Ok(())
    }

    async fn capture(&self, body: Body, expected: Option<u64>) -> (Body, Option<Arc<Capture>>) {
        let _guard = self.index.lock().await;
        if !self.budget_known.load(Ordering::SeqCst)
            || fs::create_dir_all(&self.root).await.is_err()
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
                    self.allocation.clone(),
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
        let _guard = self.index.lock().await;
        if !self.budget_known.load(Ordering::SeqCst) {
            return Err(CacheError::Unavailable);
        }
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
        let mut index = self.index.lock().await;
        if !self.budget_known.load(Ordering::SeqCst) || blob.generation != self.generation() {
            return Err(CacheError::Unavailable);
        }
        let records = entries(&self.root).await?;
        if let Some(old) = records.iter().find(|old| {
            old.scope == entry.scope
                && old.key == entry.key
                && old.version == entry.version
                && old.origin == entry.origin
                && (entry.origin == EntryOrigin::AcceptedSave || old.source == entry.source)
        }) {
            return Ok(old.clone());
        }
        entry.created = next_order(records.iter().map(|entry| entry.created))?;
        entry.accessed = next_order(records.iter().map(|entry| entry.accessed))?;
        entry.parts = blob.parts.iter().map(|part| part.size).collect();
        entry.size = blob.size;
        entry.digest = blob.digest.clone();
        entry.id = Uuid::new_v4().to_string();
        *index = None;
        persist(&self.root, &entry, blob).await?;
        Ok(entry)
    }

    async fn clear(&self) -> Result<()> {
        self.clear_files().await
    }

    async fn resize(&self, bytes: u64) -> Result<()> {
        self.resize_files(bytes).await
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
