//! Typed remote scale-set controls for App service integration scenarios.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_github::error::{Error, Result};
use ciproxy_github::scale_sets::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::organizations::Organization;

pub(super) struct Resources {
    pub(super) base_id: AtomicI64,
    pub(super) ensures: Mutex<Vec<(String, i64, Option<i64>)>>,
}
impl Resources {
    pub(super) fn new() -> Self {
        Self {
            base_id: AtomicI64::new(53),
            ensures: Mutex::new(vec![]),
        }
    }
}
#[async_trait]
impl ScaleSetApi for Resources {
    async fn ensure(&self, label: &str, group: i64, saved: Option<i64>) -> Result<i64> {
        self.ensures
            .lock()
            .unwrap()
            .push((label.to_owned(), group, saved));
        match label {
            "ciproxy" => Ok(self.base_id.load(Ordering::SeqCst)),
            "ciproxy-4vcpu" => Ok(59),
            "ciproxy-8vcpu" => Ok(61),
            _ => Err(Error::AccessUnavailable(422)),
        }
    }
    async fn open(&self, _: i64, _: &str) -> Result<Box<dyn MessageQueue>> {
        Err(Error::AccessUnavailable(501))
    }
    async fn register(&self, _: i64, _: &str) -> Result<JitRegistration> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_runner(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
    async fn remove_scale_set(&self, _: i64) -> Result<()> {
        Err(Error::AccessUnavailable(501))
    }
}

pub(super) struct ResourceFactory(pub(super) Arc<Resources>);

#[async_trait]
impl ScaleSetFactory for ResourceFactory {
    async fn for_organization(&self, _: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        Ok(self.0.clone())
    }
}
