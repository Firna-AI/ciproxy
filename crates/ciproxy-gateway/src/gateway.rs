//! Synchronous machine service consumed through dyn from blocking workers.

use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Duration;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_store_interface::gateway::{Contact, GatewayStore};
use uuid::Uuid;

use crate::authentication::hash;
use crate::boundary::{AgentEvents, Authorization, Clock, Entropy};
use crate::connections::Connections;
use crate::error::{Error, ErrorResultExt, Result};
use crate::types::{
    Delivery, Handshake, IssuedToken, MachineAction, MachineView, Presence, Session,
};

/// Machine gateway API used by the hosted HTTP and lifecycle services.
pub trait MachineGateway: Send + Sync {
    /// Issue a reusable one-hour token after checking the workspace Admin role.
    fn issue_token(&self, principal: Uuid, workspace_id: Uuid) -> Result<IssuedToken>;
    /// Revoke an install token without revoking the machines it registered.
    fn revoke_token(&self, principal: Uuid, workspace_id: Uuid, token_id: Uuid) -> Result<()>;
    /// Register or authenticate the first message on an agent WebSocket.
    fn open(&self, message: AgentMessage) -> Result<Handshake>;
    /// Store a report or deliver an authenticated lifecycle event.
    fn receive(&self, session: Session, message: AgentMessage) -> Result<()>;
    /// Check the persisted fence immediately before delivering a command.
    fn is_current(&self, session: Session) -> Result<bool>;
    /// Perform an Admin machine action and attempt immediate command delivery.
    fn manage(
        &self,
        principal: Uuid,
        workspace_id: Uuid,
        machine_id: Uuid,
        action: MachineAction,
    ) -> Result<Delivery>;
    /// Route a lifecycle command from a trusted internal service to this workspace machine.
    fn send(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: ControlMessage,
    ) -> Result<Delivery>;
    /// Read machines and mark health old at sixty seconds without contact.
    fn machines(&self, workspace_id: Uuid) -> Result<Vec<MachineView>>;
}

/// Concrete composition of the injected gateway boundaries.
pub struct Gateway {
    pub(crate) store: Arc<dyn GatewayStore>,
    pub(crate) authorization: Arc<dyn Authorization>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) entropy: Arc<dyn Entropy>,
    pub(crate) connections: Arc<dyn Connections>,
    pub(crate) events: Arc<dyn AgentEvents>,
    operation: Mutex<()>,
}

impl Gateway {
    /// Construct at a composition root, then consume through dyn MachineGateway.
    pub fn new(
        store: Arc<dyn GatewayStore>,
        authorization: Arc<dyn Authorization>,
        clock: Arc<dyn Clock>,
        entropy: Arc<dyn Entropy>,
        connections: Arc<dyn Connections>,
        events: Arc<dyn AgentEvents>,
    ) -> Self {
        Self {
            store,
            authorization,
            clock,
            entropy,
            connections,
            events,
            operation: Mutex::new(()),
        }
    }

    fn operation(&self) -> Result<MutexGuard<'_, ()>> {
        match self.operation.lock() {
            Ok(guard) => Ok(guard),
            Err(_) => Err(Error::internal_message("gateway operation poisoned")),
        }
    }

    fn require_current(&self, session: Session) -> Result<()> {
        match self
            .store
            .machine_session(session.workspace_id, session.machine_id)
            .internal()?
        {
            Some(id) if id == session.id => Ok(()),
            Some(_) => Err(Error::Replaced),
            None => Err(Error::Unauthorized),
        }
    }
}

impl MachineGateway for Gateway {
    fn issue_token(&self, principal: Uuid, workspace_id: Uuid) -> Result<IssuedToken> {
        self.authorization.require_admin(principal, workspace_id)?;
        let at = self.clock.now();
        let id = self.entropy.identifier()?;
        let token = self.entropy.secret()?;
        self.store
            .issue_install_token(workspace_id, id, &hash(&token), at)
            .internal()?;
        Ok(IssuedToken {
            id,
            token,
            expires_at: at + Duration::hours(1),
        })
    }

    fn revoke_token(&self, principal: Uuid, workspace_id: Uuid, token_id: Uuid) -> Result<()> {
        self.authorization.require_admin(principal, workspace_id)?;
        self.store
            .revoke_install_token(workspace_id, token_id, self.clock.now())
            .internal()
    }

    fn open(&self, message: AgentMessage) -> Result<Handshake> {
        let _operation = self.operation()?;
        self.authenticate(message)
    }

    fn receive(&self, session: Session, message: AgentMessage) -> Result<()> {
        match message {
            AgentMessage::Report { report } => {
                let _operation = self.operation()?;
                self.require_current(session)?;
                self.store
                    .record_contact(
                        session.workspace_id,
                        session.machine_id,
                        &Contact {
                            session_id: session.id,
                            report,
                            at: self.clock.now(),
                        },
                    )
                    .internal()
            }
            AgentMessage::Register { .. } | AgentMessage::Hello { .. } => {
                Err(Error::UnexpectedMessage)
            }
            message => {
                self.require_current(session)?;
                self.store
                    .record_activity(
                        session.workspace_id,
                        session.machine_id,
                        session.id,
                        self.clock.now(),
                    )
                    .internal()?;
                self.events.receive(session, message)
            }
        }
    }

    fn is_current(&self, session: Session) -> Result<bool> {
        Ok(self
            .store
            .machine_session(session.workspace_id, session.machine_id)
            .internal()?
            == Some(session.id))
    }

    fn manage(
        &self,
        principal: Uuid,
        workspace_id: Uuid,
        machine_id: Uuid,
        action: MachineAction,
    ) -> Result<Delivery> {
        let _operation = self.operation()?;
        self.manage_machine(principal, workspace_id, machine_id, action)
    }

    fn send(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: ControlMessage,
    ) -> Result<Delivery> {
        let _operation = self.operation()?;
        match &message {
            ControlMessage::StartRunner { .. }
            | ControlMessage::StopRunner { .. }
            | ControlMessage::JobFacts { .. }
            | ControlMessage::FetchImage { .. } => (),
            _ => return Err(Error::UnexpectedMessage),
        }
        self.deliver(workspace_id, machine_id, message)
    }

    fn machines(&self, workspace_id: Uuid) -> Result<Vec<MachineView>> {
        let now = self.clock.now();
        Ok(self
            .store
            .machine_snapshots(workspace_id)
            .internal()?
            .into_iter()
            .map(|snapshot| {
                let online = snapshot
                    .machine
                    .last_contact_at
                    .is_some_and(|at| now < at + Duration::seconds(60));
                MachineView {
                    snapshot,
                    presence: if online {
                        Presence::Online
                    } else {
                        Presence::Offline
                    },
                }
            })
            .collect())
    }
}
