//! Root-side vsock channels. A connection stays bound to one locally owned VM.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use async_trait::async_trait;
use ciproxy_protocol::guest::AgentToGuest;
use tokio::sync::{Mutex, mpsc, watch};
use tokio::task::JoinHandle;

use crate::boundaries::GuestEvent;
use crate::error::{Error, ErrorResultExt, Result};
use crate::guest_transport::{GuestTransport, UnixGuestTransport};
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
    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool>;
}

pub(crate) struct Configuration {
    pub(crate) pending: Option<AgentToGuest>,
    pub(crate) received: Option<bool>,
}

impl Configuration {
    pub(crate) fn new(pending: Option<AgentToGuest>) -> Self {
        Self {
            pending,
            received: None,
        }
    }
    pub(crate) fn lost(&self) -> bool {
        self.received == Some(false) && self.pending.is_none()
    }
}

struct Channel {
    sender: mpsc::Sender<AgentToGuest>,
    configuration: Arc<Mutex<Configuration>>,
    shutdown: watch::Sender<bool>,
    task: JoinHandle<()>,
}

pub(crate) struct LinuxGuests {
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) events: mpsc::Sender<GuestEvent>,
    channels: Mutex<HashMap<String, Channel>>,
    pub(crate) transport: Arc<dyn GuestTransport>,
}

impl LinuxGuests {
    pub(crate) fn new(
        layout: Layout,
        commands: Arc<dyn Commands>,
        events: mpsc::Sender<GuestEvent>,
    ) -> Self {
        Self {
            commands,
            events,
            channels: Mutex::new(HashMap::new()),
            transport: Arc::new(UnixGuestTransport { layout }),
        }
    }
}

#[async_trait]
impl Guests for LinuxGuests {
    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool> {
        let channels = self.channels.lock().await;
        let channel = channels.get(&vm.local_id).ok_or(Error::Disconnected)?;
        Ok(channel.configuration.lock().await.lost())
    }
    async fn attach(&self, vm: &VmRecord, config: Option<AgentToGuest>) -> Result<()> {
        let mut channels = self.channels.lock().await;
        if channels.contains_key(&vm.local_id) {
            return Ok(());
        }
        let (sender, receiver) = mpsc::channel(16);
        let (shutdown, stop_requested) = watch::channel(false);
        let initial = config.is_some();
        let configuration = Arc::new(Mutex::new(Configuration::new(config)));
        let bridge = GuestBridge {
            initial: AtomicBool::new(initial),
            configuration: configuration.clone(),
            transport: self.transport.clone(),
            commands: self.commands.clone(),
            events: self.events.clone(),
            vm: vm.clone(),
        };
        let task = tokio::spawn(async move {
            bridge.run(receiver, stop_requested).await;
        });
        channels.insert(
            vm.local_id.clone(),
            Channel {
                sender,
                configuration,
                shutdown,
                task,
            },
        );
        Ok(())
    }

    async fn detach(&self, vm: &VmRecord) -> Result<()> {
        let channel = { self.channels.lock().await.remove(&vm.local_id) };
        if let Some(channel) = channel {
            channel.task.abort();
            let _ = channel.task.await;
        }
        Ok(())
    }

    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        let sender = {
            let channels = self.channels.lock().await;
            let channel = channels.get(&vm.local_id).ok_or(Error::Disconnected)?;
            if *message == AgentToGuest::Shutdown {
                channel.shutdown.send_if_modified(|pending| {
                    if *pending {
                        false
                    } else {
                        *pending = true;
                        true
                    }
                });
                return Ok(());
            }
            if matches!(message, AgentToGuest::ConfigureRunner { .. }) {
                channel.configuration.lock().await.pending = Some(message.clone());
            }
            channel.sender.clone()
        };
        sender
            .send(message.clone())
            .await
            .context("send guest configuration")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/vsock_tests.rs"]
mod vsock_tests;

#[cfg(test)]
#[path = "_tests_/configuration_delivery_tests.rs"]
mod configuration_delivery_tests;

#[cfg(test)]
#[path = "_tests_/first_handoff_tests.rs"]
mod first_handoff_tests;

#[cfg(test)]
#[path = "_tests_/initial_handoff_tests.rs"]
mod initial_handoff_tests;
