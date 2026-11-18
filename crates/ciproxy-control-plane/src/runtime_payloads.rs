//! Byte-bounded Worker text remains charged through queued and in-flight work.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use ciproxy_protocol::identifiers::JobId;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const WORKER_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) struct PayloadId(u64);

pub(crate) struct WorkerJob {
    pub(crate) id: JobId,
    pub(crate) permit: Option<Arc<OwnedSemaphorePermit>>,
}

impl WorkerJob {
    pub(crate) fn direct(id: JobId) -> Self {
        Self { id, permit: None }
    }
}

struct StoredPayload {
    text: Box<str>,
    permit: OwnedSemaphorePermit,
}

#[derive(Default)]
struct State {
    entries: HashMap<PayloadId, StoredPayload>,
    next: u64,
    closed: bool,
}

pub(crate) struct WorkerPayloads {
    state: Mutex<State>,
    budget: Arc<Semaphore>,
}

impl WorkerPayloads {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(State::default()),
            budget: Arc::new(Semaphore::new(WORKER_PAYLOAD_BYTES)),
        }
    }

    /// Serialize producer admission with closure before any inbox task can start.
    pub(crate) fn admission(&self) -> Option<Admission<'_>> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.closed {
            return None;
        }
        Some(Admission {
            state,
            budget: &self.budget,
        })
    }

    /// Move the byte permit with the text instead of returning capacity on dequeue.
    pub(crate) fn take(&self, id: PayloadId) -> Option<WorkerJob> {
        let stored = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .remove(&id)?;
        Some(WorkerJob {
            id: JobId(stored.text.into_string()),
            permit: Some(Arc::new(stored.permit)),
        })
    }

    /// Prevent racing inserts before dropping queued text and stopping its consumer.
    pub(crate) fn close(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.closed = true;
        state.entries.clear();
        self.budget.close();
    }
}

pub(crate) struct Admission<'a> {
    state: MutexGuard<'a, State>,
    budget: &'a Arc<Semaphore>,
}

impl Admission<'_> {
    pub(crate) fn insert(&mut self, id: JobId) -> Option<PayloadId> {
        let bytes = u32::try_from(id.0.len()).ok()?;
        let permit = self.budget.clone().try_acquire_many_owned(bytes).ok()?;
        let mut token = PayloadId(self.state.next);
        while self.state.entries.contains_key(&token) {
            token.0 = token.0.wrapping_add(1);
        }
        self.state.next = token.0.wrapping_add(1);
        self.state.entries.insert(
            token,
            StoredPayload {
                text: id.0.into_boxed_str(),
                permit,
            },
        );
        Some(token)
    }
}
