//! Task ownership survives cancellation of the future that joins a worker.

use tokio::task::JoinHandle;

pub(crate) struct OwnedTask<T>(pub(crate) JoinHandle<T>);

impl<T> Drop for OwnedTask<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
