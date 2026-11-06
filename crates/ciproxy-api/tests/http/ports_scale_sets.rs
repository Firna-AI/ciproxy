//! Controlled scale-set resources, separate from identity and installation facts.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::organizations::Organization;

use super::ports::{Factory, Remote};

#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(
        &self,
        _: &Organization,
    ) -> ciproxy_github::error::Result<Arc<dyn ScaleSetApi>> {
        Ok(self.0.clone())
    }
}
#[async_trait]
impl ScaleSetApi for Remote {
    async fn find_runner(&self, _: &str) -> ciproxy_github::error::Result<Option<i64>> {
        Ok(None)
    }
    async fn ensure(
        &self,
        name: &str,
        group: i64,
        saved: Option<i64>,
    ) -> ciproxy_github::error::Result<i64> {
        if self.fail_ensure.swap(false, Ordering::SeqCst) {
            return Err(ciproxy_github::error::Error::AccessUnavailable(503));
        }
        let gate = self.ensure_gate.lock().unwrap().take();
        if let Some(gate) = gate {
            gate.entered.notify_one();
            gate.release.notified().await;
        }
        if let Some(id) = saved {
            return Ok(id);
        }
        Ok(*self
            .sets
            .lock()
            .unwrap()
            .entry((name.to_owned(), group))
            .or_insert_with(|| self.next_id.fetch_add(1, Ordering::SeqCst)))
    }
    async fn remove_scale_set(&self, id: i64) -> ciproxy_github::error::Result<()> {
        if self.fail_delete.swap(false, Ordering::SeqCst) {
            return Err(ciproxy_github::error::Error::AccessUnavailable(503));
        }
        self.events.lock().unwrap().push(format!("delete:{id}"));
        self.sets.lock().unwrap().retain(|_, value| *value != id);
        Ok(())
    }
    async fn rename_scale_set(&self, id: i64, name: &str) -> ciproxy_github::error::Result<()> {
        let mut sets = self.sets.lock().unwrap();
        let previous = sets
            .iter()
            .find(|(_, value)| **value == id)
            .map(|(key, _)| key.clone());
        if let Some((previous, group)) = previous {
            sets.remove(&(previous, group));
            sets.insert((name.to_owned(), group), id);
        }
        Ok(())
    }
    async fn open(&self, _: i64, _: &str) -> ciproxy_github::error::Result<Box<dyn MessageQueue>> {
        Err(ciproxy_github::error::Error::InstallationPending)
    }
    async fn register(&self, _: i64, _: &str) -> ciproxy_github::error::Result<JitRegistration> {
        Err(ciproxy_github::error::Error::InstallationPending)
    }
    async fn remove_runner(&self, _: i64) -> ciproxy_github::error::Result<()> {
        Ok(())
    }
}
