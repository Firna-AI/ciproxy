//! The disk boundary consumed by the cache service.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::scope::Scope;

use super::capture::{Capture, Captured};

/// Immutable local entry metadata.
#[derive(Clone, Deserialize, Serialize)]
pub struct Entry {
    /// Opaque disk object identifier.
    pub id: String,
    /// Original GitHub key.
    pub key: String,
    /// Original GitHub archive version.
    pub version: String,
    pub(crate) scope: Scope,
    /// Physical archive size, measured while writing.
    pub size: u64,
    /// Digest measured while writing, used to detect disk corruption.
    pub digest: String,
    /// Ordered block lengths, or empty when this entry has a single blob.
    #[serde(default)]
    pub parts: Vec<u64>,
    /// Persisted creation order, independent of the report clock's precision.
    pub created: u64,
    /// Persisted local access order used for eviction.
    pub accessed: u64,
}

/// Lookup assembled from trusted scopes and runner-supplied keys only.
pub struct Lookup {
    pub(crate) scopes: Vec<Scope>,
    /// Keys in their requested order.
    pub keys: Vec<String>,
    /// Archive version.
    pub version: String,
}

/// An archive read in bounded chunks without retaining disk handles.
#[derive(Clone)]
pub enum ReadSource {
    /// An immutable published archive.
    Entry(Arc<Entry>),
    /// A complete temporary archive.
    Capture(Arc<Captured>),
}

/// Persistent cache storage and bounded transient capture.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Recover orphaned files once before this storage starts serving requests.
    async fn recover(&self) -> Result<()>;
    /// Find an entry in branch-first exact/prefix key order.
    async fn lookup(&self, lookup: &Lookup) -> Result<Option<Entry>>;
    /// Verify and open a local archive.
    async fn verify(&self, entry: &Entry) -> Result<()>;
    /// Record a local entry access for least-recently-used eviction.
    async fn touch(&self, entry: &Entry) -> Result<()>;
    /// Open a complete transient capture without copying it into memory.
    async fn read(&self, source: &ReadSource, offset: u64, length: usize) -> Result<Bytes>;
    /// Snapshot the current clear generation.
    fn generation(&self) -> u64;
    /// Tee a stream to bounded transient disk without breaking its consumer.
    async fn capture(&self, body: Body, expected: Option<u64>) -> (Body, Option<Arc<Capture>>);
    /// Form one ordered archive from captured blocks without a second disk copy.
    async fn combine(&self, chunks: Vec<Arc<Captured>>) -> Result<Arc<Captured>>;
    /// Publish a captured archive if no clear occurred since capture began.
    async fn publish(&self, entry: Entry, blob: &Captured) -> Result<Entry>;
    /// Clear cache entries and invalidate pending publications.
    async fn clear(&self) -> Result<()>;
    /// Change the byte budget and evict old entries as necessary.
    async fn resize(&self, bytes: u64) -> Result<()>;
    /// Measured occupied bytes and configured capacity.
    async fn usage(&self) -> Result<(u64, u64)>;
    /// Read the machine-owned report data.
    async fn read_metrics(&self) -> Result<Vec<u8>>;
    /// Atomically replace the machine-owned report data.
    async fn write_metrics(&self, bytes: &[u8]) -> Result<()>;
}
