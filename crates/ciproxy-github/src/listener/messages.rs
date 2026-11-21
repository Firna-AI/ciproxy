//! Scale-set job decoding that preserves UUIDs and assignment timestamps.

use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use scaleset::{JobMessage, ScaleSetMessage};
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::persistence::session::ListenerContext;

use super::DurableListener;

impl DurableListener {
    pub(super) async fn persist(
        &self,
        context: &ListenerContext,
        message: &ScaleSetMessage,
    ) -> Result<()> {
        for message in &message.jobs {
            let Some(info) = message.job() else {
                tracing::warn!("GitHub sent an unknown scale-set message type");
                continue;
            };
            let (phase, result, runner_github_id) = match message {
                JobMessage::Available(_) => (DemandPhase::Available, None, None),
                JobMessage::Assigned(_) => (DemandPhase::Assigned, None, None),
                JobMessage::Started(started) => {
                    (DemandPhase::Started, None, Some(started.runner_id.get()))
                }
                JobMessage::Completed(completed) => (
                    DemandPhase::Completed,
                    Some(completed.result.clone()),
                    completed.runner_id.map(|id| id.get()),
                ),
                _ => continue,
            };
            let assigned_at = info
                .scale_set_assign_time
                .map(|at| at.to_string().parse())
                .transpose()
                .internal()?;
            if phase != DemandPhase::Available && assigned_at.is_none() {
                return Err(Error::MissingAssignmentTime);
            }
            let repository = format!("{}/{}", info.owner_name, info.repository_name);
            let facts = if let Some(facts) = self
                .store
                .facts(
                    context.organization.workspace_id,
                    context.organization.id,
                    info.workflow_run_id,
                )
                .await?
            {
                facts
            } else {
                self.rest
                    .run_facts(
                        context.organization.installation_id,
                        &repository,
                        info.workflow_run_id,
                    )
                    .await?
            };
            let job = GithubJob {
                id: Uuid::parse_str(&info.job_id).internal()?,
                organization_id: context.organization.id,
                scale_set_id: context.scale_set.id,
                label_id: context.scale_set.label_id,
                actions_job_id: None,
                request_id: info.runner_request_id.get(),
                run_id: info.workflow_run_id,
                repository,
                name: info.job_display_name.clone(),
                workflow_ref: info.job_workflow_ref.clone(),
                phase,
                result,
                assigned_at,
                started_at: info
                    .runner_assign_time
                    .map(|at| at.to_string().parse())
                    .transpose()
                    .internal()?,
                completed_at: info
                    .finish_time
                    .map(|at| at.to_string().parse())
                    .transpose()
                    .internal()?,
                runner_github_id,
                facts,
            };
            self.store
                .save(context.organization.workspace_id, job)
                .await?;
        }
        Ok(())
    }
}
