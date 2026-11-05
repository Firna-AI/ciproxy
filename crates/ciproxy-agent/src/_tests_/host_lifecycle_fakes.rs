//! A pending runtime download has explicit readiness and completion barriers.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::{Mutex, oneshot};

use crate::error::Result;
use crate::host_tools::Tools;

pub(crate) struct HeldTools {
    pub(crate) delegate: Arc<dyn Tools>,
    pub(crate) started: Mutex<Option<oneshot::Sender<()>>>,
    pub(crate) release: Mutex<Option<oneshot::Receiver<()>>>,
    pub(crate) complete: Arc<AtomicBool>,
}

#[async_trait]
impl Tools for HeldTools {
    async fn prepare(&self) -> Result<()> {
        self.delegate.prepare().await?;
        self.started.lock().await.take().unwrap().send(()).unwrap();
        let release = self.release.lock().await.take().unwrap();
        release.await.unwrap();
        self.complete.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn remove(&self) -> Result<()> {
        self.delegate.remove()
    }
}
