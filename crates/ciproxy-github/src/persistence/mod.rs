//! Blocking-worker adapters from dyn Store to narrow async service seams.

pub mod installation;
pub mod mutation;
pub mod session;

use std::sync::Arc;

use ciproxy_store_interface::error;
use ciproxy_store_interface::store::Store;

use crate::error::{Result, ResultExt};
use crate::runtime::Runtime;

/// The only store composition used by this crate's asynchronous services.
pub struct StoreAdapter {
    pub(super) store: Arc<dyn Store>,
    pub(super) runtime: Arc<dyn Runtime>,
}

impl StoreAdapter {
    /// Bind a shared hosted store without putting blocking queries on Tokio.
    pub fn new(store: Arc<dyn Store>, runtime: Arc<dyn Runtime>) -> Self {
        Self { store, runtime }
    }
}

pub(super) async fn query<T: Send + 'static>(
    store: Arc<dyn Store>,
    action: impl FnOnce(&dyn Store) -> error::Result<T> + Send + 'static,
) -> Result<T> {
    tokio::task::spawn_blocking(move || action(&*store))
        .await
        .internal()?
        .internal()
}
