//! Image-bound Docker policy reaches the machine before trusted job facts.

use ciproxy_gateway::types::Delivery;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use uuid::Uuid;

use crate::docker_budget::policy;
use crate::error::{ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn send_docker_budget(
        &self,
        input: &RoutingSnapshot,
        machine_id: Uuid,
    ) -> Result<bool> {
        let message = ControlMessage::DockerBudget {
            command_id: CommandId(self.clock.uuid().to_string()),
            budget: policy(input, machine_id),
        };
        let workspace_id = input.workspace_id;
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
