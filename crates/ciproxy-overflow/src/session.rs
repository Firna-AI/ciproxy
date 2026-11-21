//! Blocking-worker access to synchronous PostgreSQL session queries.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::error;
use ciproxy_store_interface::overflow::store::{OverflowSession, OverflowStore};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};

pub(crate) struct Session {
    inner: Arc<Mutex<Box<dyn OverflowSession>>>,
}

impl Session {
    pub(crate) async fn acquire(
        store: Arc<dyn OverflowStore>,
        workspace_id: Uuid,
        wait: bool,
    ) -> Result<Self> {
        let session = spawn_blocking(move || {
            if wait {
                return store.lock_overflow(workspace_id).map(Some);
            }
            store.try_lock_overflow(workspace_id)
        })
        .await
        .internal()?
        .internal()?
        .ok_or(Error::Busy)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(session)),
        })
    }

    pub(crate) async fn call<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut dyn OverflowSession) -> error::Result<T> + Send + 'static,
    ) -> Result<T> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let mut session = match inner.lock() {
                Ok(session) => session,
                Err(_) => return Err(error::Error::Conflict),
            };
            operation(&mut **session)
        })
        .await
        .internal()?
        .internal()
    }
}
