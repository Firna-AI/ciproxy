//! Capture machine delivery without requiring an unrelated policy command.

use std::sync::{Arc, Mutex};

use ciproxy_gateway::error::{Error, Result};
use ciproxy_gateway::gateway::MachineGateway;
use ciproxy_gateway::types::{
    Delivery, Handshake, IssuedToken, MachineAction, MachineView, Session,
};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use uuid::Uuid;

pub(super) struct Capture {
    pub(super) messages: Mutex<Vec<ControlMessage>>,
    offline: bool,
}

impl Capture {
    pub(super) fn new(offline: bool) -> Arc<Self> {
        Arc::new(Self {
            messages: Mutex::new(Vec::new()),
            offline,
        })
    }
}

impl MachineGateway for Capture {
    fn send(&self, _: Uuid, _: Uuid, message: ControlMessage) -> Result<Delivery> {
        self.messages.lock().unwrap().push(message);
        Ok(if self.offline {
            Delivery::Offline
        } else {
            Delivery::Connected
        })
    }
    fn issue_token(&self, _: Uuid, _: Uuid) -> Result<IssuedToken> {
        Err(Error::UnexpectedMessage)
    }
    fn revoke_token(&self, _: Uuid, _: Uuid, _: Uuid) -> Result<()> {
        Err(Error::UnexpectedMessage)
    }
    fn open(&self, _: AgentMessage) -> Result<Handshake> {
        Err(Error::UnexpectedMessage)
    }
    fn receive(&self, _: Session, _: AgentMessage) -> Result<()> {
        Err(Error::UnexpectedMessage)
    }
    fn is_current(&self, _: Session) -> Result<bool> {
        Err(Error::UnexpectedMessage)
    }
    fn manage(&self, _: Uuid, _: Uuid, _: Uuid, _: MachineAction) -> Result<Delivery> {
        Err(Error::UnexpectedMessage)
    }
    fn machines(&self, _: Uuid) -> Result<Vec<MachineView>> {
        Err(Error::UnexpectedMessage)
    }
}
