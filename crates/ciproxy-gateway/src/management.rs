//! Desired machine state and immediate local command delivery.

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::CommandId;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::Gateway;
use crate::types::{Delivery, MachineAction, Session};

impl Gateway {
    pub(crate) fn deliver(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: ControlMessage,
    ) -> Result<Delivery> {
        let Some(id) = self
            .store
            .machine_session(workspace_id, machine_id)
            .internal()?
        else {
            return Err(Error::NotFound);
        };
        self.connections.send(
            Session {
                workspace_id,
                machine_id,
                id,
            },
            message,
        )
    }

    pub(crate) fn manage_machine(
        &self,
        principal: Uuid,
        workspace_id: Uuid,
        machine_id: Uuid,
        action: MachineAction,
    ) -> Result<Delivery> {
        self.authorization.require_admin(principal, workspace_id)?;
        let id = self
            .store
            .machine_session(workspace_id, machine_id)
            .internal()?;
        let command_id = CommandId(self.entropy.identifier()?.to_string());
        let message = match action {
            MachineAction::Drain => {
                self.store
                    .set_machine_draining(workspace_id, machine_id, true)
                    .internal()?;
                ControlMessage::Drain { command_id }
            }
            MachineAction::Resume => {
                self.store
                    .set_machine_draining(workspace_id, machine_id, false)
                    .internal()?;
                ControlMessage::Resume { command_id }
            }
            MachineAction::Remove => {
                self.store
                    .revoke_machine(workspace_id, machine_id, self.clock.now())
                    .internal()?;
                ControlMessage::Remove { command_id }
            }
            MachineAction::ClearCache => ControlMessage::ClearCache { command_id },
        };
        let Some(id) = id else {
            return Ok(Delivery::Offline);
        };
        self.connections.send(
            Session {
                workspace_id,
                machine_id,
                id,
            },
            message,
        )
    }
}
