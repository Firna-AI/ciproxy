//! Record trusted GitHub job assignment on the actual Boat runner placement.

use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::runners::RunnerState;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::MachineRuntime;

impl MachineRuntime {
    pub(crate) async fn route_boat_job(
        &self,
        input: &RoutingSnapshot,
        job: &GithubJob,
    ) -> Result<()> {
        if job.phase < DemandPhase::Started {
            return Ok(());
        }
        let Some(intent) =
            input.boat_intents.iter().find(|intent| {
                intent.organization_id == job.organization_id
                    && intent.scale_set_id == job.scale_set_id
                    && job.runner_github_id.is_some()
                    && intent.github_id == job.runner_github_id
                    && input.overflow.starts.iter().any(|record| {
                        record.runner_id == intent.runner_id && record.ends_at.is_none()
                    })
            })
        else {
            return Ok(());
        };
        if job.phase == DemandPhase::Started {
            let store = self.store.clone();
            let workspace = input.workspace_id;
            let organization = job.organization_id;
            let runner = intent.runner_id;
            let id = job.runner_github_id;
            tokio::task::spawn_blocking(move || {
                store.update_runner(workspace, organization, runner, RunnerState::Busy, id)
            })
            .await
            .internal()?
            .internal()?;
        }
        if job.phase == DemandPhase::Completed
            && let Err(error) = self.cleanup_boat(input.workspace_id).await
        {
            tracing::warn!(%error,"Boat completion cleanup will retry");
        }
        Ok(())
    }
}
