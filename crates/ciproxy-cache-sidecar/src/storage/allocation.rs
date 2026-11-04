//! Shared allocation from measured incoming bytes and a cached eviction index.

use std::cmp::Reverse;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::error::Result;

use super::capture::Reservation;
use super::index::{entries, remove};
use super::interface::Entry;

#[async_trait]
pub(super) trait Allocation: Send + Sync {
    /// Reclaim and charge space for bytes that have already arrived.
    async fn claim(&self, reservation: &Reservation, bytes: u64) -> Result<bool>;
    /// Reclaim under an already-held storage lock, also used during resize.
    async fn reclaim(&self, index: &mut Option<Vec<Entry>>, bytes: u64) -> Result<()>;
}

pub(super) struct DiskAllocation {
    root: PathBuf,
    used: Arc<AtomicU64>,
    capacity: Arc<AtomicU64>,
    index: Arc<Mutex<Option<Vec<Entry>>>>,
}

impl DiskAllocation {
    pub fn new(
        root: PathBuf,
        used: Arc<AtomicU64>,
        capacity: Arc<AtomicU64>,
        index: Arc<Mutex<Option<Vec<Entry>>>>,
    ) -> Self {
        Self {
            root,
            used,
            capacity,
            index,
        }
    }

    fn fits(&self, bytes: u64) -> bool {
        self.used
            .load(Ordering::SeqCst)
            .checked_add(bytes)
            .is_some_and(|next| next <= self.capacity.load(Ordering::SeqCst))
    }
}

#[async_trait]
impl Allocation for DiskAllocation {
    async fn claim(&self, reservation: &Reservation, bytes: u64) -> Result<bool> {
        let mut index = self.index.lock().await;
        if !reservation.active() || bytes > self.capacity.load(Ordering::SeqCst) {
            return Ok(false);
        }
        self.reclaim(&mut index, bytes).await?;
        Ok(reservation.claim(bytes))
    }

    async fn reclaim(&self, index: &mut Option<Vec<Entry>>, bytes: u64) -> Result<()> {
        if self.fits(bytes) {
            return Ok(());
        }
        if index.is_none() {
            let mut records = entries(&self.root).await?;
            records.sort_by_key(|entry| Reverse((entry.accessed, entry.created, entry.id.clone())));
            *index = Some(records);
        }
        while !self.fits(bytes) {
            let Some(entry) = index.as_mut().and_then(Vec::pop) else {
                break;
            };
            if let Err(error) = remove(&self.root, &entry).await {
                *index = None;
                return Err(error);
            }
            self.used.fetch_sub(entry.size, Ordering::SeqCst);
        }
        Ok(())
    }
}
