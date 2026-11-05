//! Private runner input and restart-safe once-only privileged launch.

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{BackendWriteFileRequest, ProviderRef};
use serde::Serialize;
use uuid::Uuid;

use crate::api::{StartOutcome, StartRequest};
use crate::crypto::{KeyContext, SecretPurpose};
use crate::error::{Error, Result, ResultExt};
use crate::maintenance::MaintenanceCapabilities;
use crate::provider::registration_root;
use crate::secrets::SecretBytes;
use crate::session::Session;

use super::OverflowService;

#[derive(Serialize)]
struct LaunchInput {
    configure: AgentToGuest,
    exit_report_url: Secret,
    exit_report_token: Secret,
}

impl OverflowService {
    pub(super) fn launch_input(
        &self,
        request: &StartRequest,
        token: Secret,
    ) -> Result<SecretBytes> {
        let input = LaunchInput {
            configure: AgentToGuest::ConfigureRunner {
                protocol_version: ProtocolVersion::CURRENT,
                runner_id: RunnerId(request.runner_id.to_string()),
                organization_id: request.action.organization_id.clone(),
                registration: request.registration.clone(),
                cache: None,
            },
            exit_report_url: self.config.exit_report_url.clone(),
            exit_report_token: token,
        };
        Ok(SecretBytes::new(serde_json::to_vec(&input).internal()?))
    }

    pub(super) async fn launch(
        &self,
        session: &Session,
        workspace_id: Uuid,
        backend: &MaintenanceCapabilities,
        record: &mut StartRecord,
    ) -> Result<StartOutcome> {
        let sandbox =
            ProviderRef::new(record.sandbox.as_ref().ok_or(Error::OwnershipUnknown)?.id());
        if record.phase == StartPhase::Preparing {
            let Some(authority) = self
                .launch_authority(session, workspace_id, record.key_id)
                .await?
            else {
                return self
                    .reconcile_retired(session, workspace_id, backend, record.clone())
                    .await;
            };
            let delivery = record.delivery_id;
            let input = session
                .call(move |session| session.encrypted_launch(delivery))
                .await?
                .ok_or(Error::DeliveryConflict)?;
            let bytes = self
                .dependencies
                .cipher
                .open(
                    KeyContext {
                        workspace_id,
                        id: delivery,
                        purpose: SecretPurpose::Launch,
                    },
                    input.0,
                )
                .await?;
            record.phase = StartPhase::Writing;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
            let bootstrap = authority
                .files()
                .write_file(BackendWriteFileRequest {
                    sandbox_provider_ref: sandbox.clone(),
                    root: "/workspace".to_owned(),
                    path: format!(".ciproxy-bootstrap-{}", delivery.simple()),
                    bytes: Vec::new(),
                })
                .await;
            self.confirm_write(session, backend, record, bootstrap)
                .await?;
            authority
                .launcher()
                .prepare(sandbox.clone(), delivery)
                .await?;
            let Some(authority) = self
                .launch_authority(session, workspace_id, record.key_id)
                .await?
            else {
                return self
                    .reconcile_retired(session, workspace_id, backend, record.clone())
                    .await;
            };
            let write = authority
                .files()
                .write_file(BackendWriteFileRequest {
                    sandbox_provider_ref: sandbox.clone(),
                    root: registration_root(delivery),
                    path: "registration.json".to_owned(),
                    bytes: bytes.expose().to_vec(),
                })
                .await;
            self.confirm_write(session, backend, record, write).await?;
            record.phase = StartPhase::Launching;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
        }
        if record.phase == StartPhase::Launching {
            let Some(authority) = self
                .launch_authority(session, workspace_id, record.key_id)
                .await?
            else {
                return self
                    .reconcile_retired(session, workspace_id, backend, record.clone())
                    .await;
            };
            authority
                .launcher()
                .launch(sandbox, record.delivery_id)
                .await?;
            record.phase = StartPhase::Running;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
        }
        Ok(StartOutcome::Started(record.clone()))
    }

    async fn confirm_write(
        &self,
        session: &Session,
        backend: &MaintenanceCapabilities,
        record: &mut StartRecord,
        result: sandbox_interface::Result<()>,
    ) -> Result<()> {
        match result {
            Ok(()) => Ok(()),
            Err(sandbox_interface::Error::FileWriteUnconfirmed) => {
                record.phase = StartPhase::Deleting;
                let changed = record.clone();
                session
                    .call(move |session| session.save_start(&changed))
                    .await?;
                self.delete(session, backend, record).await?;
                Err(Error::ProviderUnavailable)
            }
            result => result.internal(),
        }
    }
}
