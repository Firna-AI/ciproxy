//! Root-side vsock channels. A connection stays bound to one locally owned VM.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::guest::AgentToGuest;
use tokio::sync::{Mutex, mpsc};
use tokio::task::AbortHandle;

use crate::boundaries::{Cache, GuestEvent};
use crate::error::{Error, ErrorResultExt, Result};
use crate::layout::Layout;
use crate::process::Commands;
use crate::state::VmRecord;
use crate::vsock_stream::GuestBridge;

#[cfg_attr(test, unimock::unimock(api = GuestsMock))]
#[async_trait]
pub(crate) trait Guests: Send + Sync {
    async fn attach(&self, vm: &VmRecord, config: Option<AgentToGuest>) -> Result<()>;
    async fn detach(&self, vm: &VmRecord) -> Result<()>;
    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()>;
}

struct Channel {
    sender: mpsc::Sender<AgentToGuest>,
    task: AbortHandle,
}

pub(crate) struct LinuxGuests {
    pub(crate) layout: Layout,
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) cache: Arc<dyn Cache>,
    pub(crate) events: mpsc::Sender<GuestEvent>,
    channels: Mutex<HashMap<String, Channel>>,
}

impl LinuxGuests {
    pub(crate) fn new(
        layout: Layout,
        commands: Arc<dyn Commands>,
        cache: Arc<dyn Cache>,
        events: mpsc::Sender<GuestEvent>,
    ) -> Self {
        Self {
            layout,
            commands,
            cache,
            events,
            channels: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Guests for LinuxGuests {
    async fn attach(&self, vm: &VmRecord, config: Option<AgentToGuest>) -> Result<()> {
        let mut channels = self.channels.lock().await;
        if channels.contains_key(&vm.local_id) {
            return Ok(());
        }
        let (sender, receiver) = mpsc::channel(16);
        let bridge = GuestBridge {
            layout: self.layout.clone(),
            commands: self.commands.clone(),
            cache: self.cache.clone(),
            events: self.events.clone(),
            vm: vm.clone(),
        };
        let task = tokio::spawn(async move {
            bridge.run(config, receiver).await;
        });
        channels.insert(
            vm.local_id.clone(),
            Channel {
                sender,
                task: task.abort_handle(),
            },
        );
        Ok(())
    }

    async fn detach(&self, vm: &VmRecord) -> Result<()> {
        if let Some(channel) = self.channels.lock().await.remove(&vm.local_id) {
            channel.task.abort();
        }
        Ok(())
    }

    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        let sender = self
            .channels
            .lock()
            .await
            .get(&vm.local_id)
            .map(|channel| channel.sender.clone())
            .ok_or(Error::Disconnected)?;
        sender
            .send(message.clone())
            .await
            .context("send guest configuration")?;
        Ok(())
    }
}
