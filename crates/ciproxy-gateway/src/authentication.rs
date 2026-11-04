//! Token hashing, registration, and connection reconciliation.

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{CommandId, MachineId, WorkspaceId};
use ciproxy_protocol::reports::{DrainState, MachineReport};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_store_interface::gateway::{Contact, Registration};
use ciproxy_store_interface::secrets::TokenHash;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::Gateway;
use crate::types::{Handshake, Session};
use crate::versions;

pub(crate) fn hash(secret: &Secret) -> TokenHash {
    TokenHash::new(Sha256::digest(secret.expose_secret().as_bytes()).to_vec())
}

fn identifier(value: &str) -> Result<Uuid> {
    match Uuid::parse_str(value) {
        Ok(id) => Ok(id),
        Err(_) => Err(Error::InvalidMessage),
    }
}

impl Gateway {
    pub(crate) fn authenticate(&self, message: AgentMessage) -> Result<Handshake> {
        match message {
            AgentMessage::Register {
                protocol_version,
                workspace_id,
                install_token,
                report,
            } => {
                versions::accept(protocol_version)?;
                let session = Session {
                    workspace_id: identifier(&workspace_id.0)?,
                    machine_id: self.entropy.identifier()?,
                    id: self.entropy.identifier()?,
                };
                let credential = self.entropy.secret()?;
                let registration = Registration {
                    workspace_id: session.workspace_id,
                    machine_id: session.machine_id,
                    install_hash: hash(&install_token),
                    credential_id: self.entropy.identifier()?,
                    credential_hash: hash(&credential),
                    contact: Contact {
                        session_id: session.id,
                        report,
                        at: self.clock.now(),
                    },
                };
                if !self.store.register_machine(&registration).internal()? {
                    return Err(Error::Unauthorized);
                }
                Ok(Handshake {
                    session,
                    outbox: self.connections.attach(session)?,
                    replies: vec![ControlMessage::Registered {
                        workspace_id: WorkspaceId(session.workspace_id.to_string()),
                        machine_id: MachineId(session.machine_id.to_string()),
                        credential,
                    }],
                })
            }
            AgentMessage::Hello {
                protocol_version,
                workspace_id,
                machine_id,
                credential,
                report,
            } => {
                versions::accept(protocol_version)?;
                self.reconnect(
                    protocol_version,
                    workspace_id,
                    machine_id,
                    credential,
                    report,
                )
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }

    fn reconnect(
        &self,
        version: ProtocolVersion,
        workspace: WorkspaceId,
        machine: MachineId,
        credential: Secret,
        report: MachineReport,
    ) -> Result<Handshake> {
        let session = Session {
            workspace_id: identifier(&workspace.0)?,
            machine_id: identifier(&machine.0)?,
            id: self.entropy.identifier()?,
        };
        let observed_drain = report.drain;
        let contact = Contact {
            session_id: session.id,
            report,
            at: self.clock.now(),
        };
        let Some(machine) = self
            .store
            .connect_machine(
                session.workspace_id,
                session.machine_id,
                &hash(&credential),
                &contact,
            )
            .internal()?
        else {
            return Err(Error::Unauthorized);
        };
        let mut replies = vec![ControlMessage::Welcome {
            protocol_version: version,
        }];
        if machine.draining || observed_drain == DrainState::Draining {
            let command_id = CommandId(self.entropy.identifier()?.to_string());
            replies.push(if machine.draining {
                ControlMessage::Drain { command_id }
            } else {
                ControlMessage::Resume { command_id }
            });
        }
        Ok(Handshake {
            session,
            replies,
            outbox: self.connections.attach(session)?,
        })
    }
}
