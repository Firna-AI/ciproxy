//! Image-bound Docker policy reaches the machine before trusted job facts.

use ciproxy_gateway::types::Delivery;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::cache::DockerBudget;
use ciproxy_protocol::identifiers::CommandId;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn send_docker_budget(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        budget: Option<DockerBudget>,
    ) -> Result<bool> {
        let message = ControlMessage::DockerBudget {
            command_id: CommandId(self.clock.uuid().to_string()),
            budget,
        };
        let gateway = self.gateway.clone();
        match tokio::task::spawn_blocking(move || gateway.send(workspace_id, machine_id, message))
            .await
            .internal()?
        {
            Ok(Delivery::Connected) => Ok(true),
            Ok(Delivery::Offline)
            | Err(ciproxy_gateway::error::Error::NotFound)
            | Err(ciproxy_gateway::error::Error::Unauthorized) => Ok(false),
            Err(error) => Err(error).internal(),
        }
    }
}
