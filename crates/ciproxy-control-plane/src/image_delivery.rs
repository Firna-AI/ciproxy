//! Coalesce release downloads by authenticated machine connection.

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_store_interface::own_runners::RoutingSnapshot;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::{ImageRequest, MachineRuntime};

impl MachineRuntime {
    pub(crate) async fn ensure_image(&self, input: &RoutingSnapshot) -> Result<()> {
        let Some(image) = &self.image else {
            return Ok(());
        };
        for machine in &input.machines {
            let Some(report) = &machine.report else {
                continue;
            };
            if report.images.newest.as_ref() == Some(&image.image_id) {
                self.image_requests
                    .lock()
                    .await
                    .remove(&(input.workspace_id, machine.machine.id));
                continue;
            }
            let store = self.store.clone();
            let workspace_id = input.workspace_id;
            let machine_id = machine.machine.id;
            let Some(session_id) = tokio::task::spawn_blocking(move || {
                store.machine_session(workspace_id, machine_id)
            })
            .await
            .internal()?
            .internal()?
            else {
                continue;
            };
            if self
                .image_requests
                .lock()
                .await
                .get(&(workspace_id, machine_id))
                .is_some_and(|request| {
                    request.session_id == session_id && request.image_id == image.image_id
                })
            {
                continue;
            }
            let command_id = self.clock.uuid();
            let mut download = image.clone();
            if let Some(signing) = &self.image_signing {
                download.download_url =
                    signing.sign(image.download_url.expose_secret(), self.clock.now())?;
            }
            let message = ControlMessage::FetchImage {
                command_id: CommandId(command_id.to_string()),
                image: download,
            };
            let gateway = self.gateway.clone();
            let workspace_id = input.workspace_id;
            let machine_id = machine.machine.id;
            let delivery = tokio::task::spawn_blocking(move || {
                gateway.send(workspace_id, machine_id, message)
            })
            .await
            .internal()?
            .internal()?;
            if delivery == ciproxy_gateway::types::Delivery::Connected {
                self.image_requests.lock().await.insert(
                    (workspace_id, machine_id),
                    ImageRequest {
                        image_id: image.image_id.clone(),
                        session_id,
                        command_id,
                    },
                );
            }
        }
        Ok(())
    }
}
