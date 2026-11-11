//! One-shot metadata requests for the root agent, without a hosted credential.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::cache_metadata::{
    CacheMetadataAnswer, CacheMetadataResponse, CacheMetadataWork,
};
use ciproxy_protocol::identifiers::{CacheMetadataRequestId, JobId};
use tokio::sync::{Mutex, Notify, oneshot};
use tokio::time::timeout;

use crate::error::{CacheError, Result};
use crate::machine::identifier;

/// Private administration queue consumed by the root agent only.
#[async_trait]
pub(crate) trait MetadataRequests: Send + Sync {
    /// Enable the relay after host administration succeeds.
    async fn enable(&self, enabled: bool);
    /// Obtain current metadata, preserving standalone mode when disabled.
    async fn request(&self, job_id: JobId) -> Result<Option<CacheMetadataResponse>>;
    /// Wait briefly for one live request without polling in a tight loop.
    async fn next(&self) -> Option<CacheMetadataWork>;
    /// Complete one pending request. Late and duplicate answers have no effect.
    async fn answer(&self, answer: CacheMetadataAnswer);
}

#[derive(Default)]
struct Pending {
    queue: VecDeque<CacheMetadataWork>,
    answers: HashMap<CacheMetadataRequestId, oneshot::Sender<Option<CacheMetadataResponse>>>,
}

/// In-memory requests scoped to this cache-service process.
pub(crate) struct AdminMetadataRelay {
    enabled: AtomicBool,
    pending: Mutex<Pending>,
    ready: Notify,
}

impl AdminMetadataRelay {
    /// Create a disabled relay with no hosted secrets.
    pub(crate) fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            pending: Mutex::new(Pending::default()),
            ready: Notify::new(),
        }
    }
}

#[async_trait]
impl MetadataRequests for AdminMetadataRelay {
    async fn enable(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            *self.pending.lock().await = Pending::default();
        }
    }

    async fn request(&self, job_id: JobId) -> Result<Option<CacheMetadataResponse>> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let request_id = CacheMetadataRequestId(identifier());
        let (send, receive) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.queue.push_back(CacheMetadataWork {
                request_id: request_id.clone(),
                job_id,
            });
            pending.answers.insert(request_id.clone(), send);
        }
        self.ready.notify_one();
        let result = timeout(Duration::from_secs(95), receive).await;
        let mut pending = self.pending.lock().await;
        pending.answers.remove(&request_id);
        pending
            .queue
            .retain(|request| request.request_id != request_id);
        match result {
            Ok(Ok(Some(metadata))) => Ok(Some(metadata)),
            _ => Err(CacheError::Unavailable),
        }
    }

    async fn next(&self) -> Option<CacheMetadataWork> {
        loop {
            let ready = self.ready.notified();
            {
                let mut pending = self.pending.lock().await;
                while let Some(request) = pending.queue.pop_front() {
                    if pending
                        .answers
                        .get(&request.request_id)
                        .is_some_and(|send| !send.is_closed())
                    {
                        return Some(request);
                    }
                    pending.answers.remove(&request.request_id);
                }
            }
            if timeout(Duration::from_secs(5), ready).await.is_err() {
                return None;
            }
        }
    }

    async fn answer(&self, answer: CacheMetadataAnswer) {
        if let Some(send) = self.pending.lock().await.answers.remove(&answer.request_id) {
            let _ = send.send(answer.metadata);
        }
    }
}
