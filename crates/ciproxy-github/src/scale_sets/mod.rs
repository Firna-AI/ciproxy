//! Narrow scale-set and message-session boundaries backed by scaleset.

mod failure;
mod queue;
pub mod sdk;

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::organizations::Organization;
use scaleset::{MessageId, ScaleSetMessage};
use secrecy::SecretString;

use crate::error::Result;

/// A one-runner JIT registration. Debug redacts its encoded configuration.
#[derive(Debug)]
pub struct JitRegistration {
    /// The GitHub identifier to remove when the runner ends.
    pub runner_id: i64,
    /// The one-use configuration sent only to its selected runner.
    pub configuration: SecretString,
}

/// Organization-scoped API construction at the App credential boundary.
#[cfg_attr(test, unimock::unimock(api = ScaleSetFactoryMock))]
#[async_trait]
pub trait ScaleSetFactory: Send + Sync {
    /// Bind App credentials to one verified organization installation.
    async fn for_organization(&self, organization: &Organization) -> Result<Arc<dyn ScaleSetApi>>;
}

/// Administrative scale-set operations for exactly one organization.
#[cfg_attr(test, unimock::unimock(api = ScaleSetApiMock))]
#[async_trait]
pub trait ScaleSetApi: Send + Sync {
    /// Find or create one label in the dedicated group with updates disabled.
    async fn ensure(&self, label: &str, runner_group_id: i64, saved_id: Option<i64>)
    -> Result<i64>;
    /// Open one message session after the caller has acquired the store lock.
    async fn open(&self, scale_set_id: i64, owner: &str) -> Result<Box<dyn MessageQueue>>;
    /// Create the ephemeral registration for one selected runner identity.
    async fn register(&self, scale_set_id: i64, name: &str) -> Result<JitRegistration>;
    /// Delete an idle or ended runner. A busy runner returns RunnerBusy.
    async fn remove_runner(&self, runner_id: i64) -> Result<()>;
    /// Delete a scale set while App access still exists, such as a take-out.
    async fn remove_scale_set(&self, scale_set_id: i64) -> Result<()>;
}

/// An owned GitHub message session. It never acknowledges messages itself.
#[cfg_attr(test, unimock::unimock(api = MessageQueueMock))]
#[async_trait]
pub trait MessageQueue: Send + Sync {
    /// Poll with the last durably acknowledged message identifier.
    async fn poll(
        &self,
        last_message_id: Option<MessageId>,
        capacity: u32,
    ) -> Result<Option<ScaleSetMessage>>;
    /// Acquire only explicit JobAvailable requests, if GitHub sends them.
    async fn acquire(&self, requests: &[scaleset::JobRequestId]) -> Result<()>;
    /// Acknowledge a batch only after its job updates are durable.
    async fn acknowledge(&self, message_id: MessageId) -> Result<()>;
    /// Close the GitHub session before releasing its database lock.
    async fn close(&self) -> Result<()>;
}
