//! Local command mailboxes keyed by authenticated connection identifiers.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use ciproxy_protocol::agent::ControlMessage;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use unimock::unimock;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::{Delivery, Session};

/// The receiver owned by one WebSocket transport.
#[async_trait]
pub trait Outbox: Send {
    /// Wait for the next command, or the replacement of this connection.
    async fn next(&mut self) -> Option<ControlMessage>;
}

/// Command routing and connection replacement supplied to the gateway.
#[unimock(api = ConnectionsMock)]
pub trait Connections: Send + Sync {
    /// Attach one connection and close the preceding connection's mailbox.
    fn attach(&self, session: Session) -> Result<Box<dyn Outbox>>;
    /// Route only to the exact connection that passed the persisted fence.
    fn send(&self, session: Session, message: ControlMessage) -> Result<Delivery>;
    /// Detach this connection without detaching its replacement.
    fn detach(&self, session: Session) -> Result<()>;
}

type MachineKey = (Uuid, Uuid);
type Mailbox = (Uuid, UnboundedSender<ControlMessage>);

/// In-process mailboxes used by the HTTP composition root.
#[derive(Default)]
pub struct LocalConnections {
    mailboxes: Mutex<HashMap<MachineKey, Mailbox>>,
}

struct LocalOutbox(UnboundedReceiver<ControlMessage>);

#[async_trait]
impl Outbox for LocalOutbox {
    async fn next(&mut self) -> Option<ControlMessage> {
        self.0.recv().await
    }
}

impl Connections for LocalConnections {
    fn attach(&self, session: Session) -> Result<Box<dyn Outbox>> {
        let (sender, receiver) = unbounded_channel();
        match self.mailboxes.lock() {
            Ok(mut entries) => {
                entries.insert(
                    (session.workspace_id, session.machine_id),
                    (session.id, sender),
                );
                Ok(Box::new(LocalOutbox(receiver)))
            }
            Err(_) => Err(Error::internal_message("connection registry poisoned")),
        }
    }

    fn send(&self, session: Session, message: ControlMessage) -> Result<Delivery> {
        let entries = match self.mailboxes.lock() {
            Ok(entries) => entries,
            Err(_) => return Err(Error::internal_message("connection registry poisoned")),
        };
        let Some((id, sender)) = entries.get(&(session.workspace_id, session.machine_id)) else {
            return Ok(Delivery::Offline);
        };
        if *id != session.id || sender.send(message).is_err() {
            return Ok(Delivery::Offline);
        }
        Ok(Delivery::Connected)
    }

    fn detach(&self, session: Session) -> Result<()> {
        match self.mailboxes.lock() {
            Ok(mut entries) => {
                let key = (session.workspace_id, session.machine_id);
                if entries.get(&key).is_some_and(|(id, _)| *id == session.id) {
                    entries.remove(&key);
                }
                Ok(())
            }
            Err(_) => Err(Error::internal_message("connection registry poisoned")),
        }
    }
}
