//! Explicit wait completions drive async entry points without a real clock.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use crate::boundaries::Delay;

pub(crate) struct Wait {
    pub(crate) seconds: u64,
    pub(crate) release: oneshot::Sender<()>,
}

pub(crate) struct ManualDelay {
    pub(crate) delegate: Arc<dyn Delay>,
    pub(crate) requests: mpsc::UnboundedSender<Wait>,
}

#[async_trait]
impl Delay for ManualDelay {
    async fn wait(&self, seconds: u64) {
        self.delegate.wait(seconds).await;
        let (release, receive) = oneshot::channel();
        self.requests.send(Wait { seconds, release }).unwrap();
        let _ = receive.await;
    }
}
