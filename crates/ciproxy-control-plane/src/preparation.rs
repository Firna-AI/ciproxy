//! Remote assignment facts return through current saved runner and job authority.

use ciproxy_github::persistence::lease::LeaseStore;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::scale_sets::JitRegistration;
use ciproxy_protocol::agent::{ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::cache::DockerBudget;
use ciproxy_protocol::identifiers::{CommandId, RunnerId};
use ciproxy_store_interface::runtime_results::OwnRunnerResult;
use uuid::Uuid;

use crate::assignment::Assignment;
use crate::error::{Error, ErrorResultExt, Result};
use crate::runtime::MachineRuntime;
use crate::runtime_tasks::Work;

impl MachineRuntime {
    /// Retain one job, both job IDs, saved authority, and computed machine policy.
    pub(crate) async fn prepare_job(
        &self,
        workspace: Uuid,
        assignment: &Assignment<'_>,
        machine: Uuid,
        budget: Option<DockerBudget>,
    ) -> Result<()> {
        let Some(job_ids) = assignment.job_ids() else {
            return Ok(());
        };
        let runner_id = assignment.runner.runner.id;
        let job = assignment.job;
        let store = self.store.clone();
        let expected =
            tokio::task::spawn_blocking(move || store.own_runner_guard(workspace, runner_id))
                .await
                .internal()?
                .internal()?
                .ok_or(Error::StateChanged)?;
        let job = job.clone();
        let runtime = self.worker();
        self.dispatch(Work::Preparation(job.id), async move {
            let target = ListenerTarget {
                workspace_id: workspace,
                organization_id: job.organization_id,
                scale_set_id: job.scale_set_id,
            };
            let jit = JitRegistration {
                runner_id: job.runner_github_id.ok_or(Error::StateChanged)?,
                configuration: "".into(),
            };
            let outcome = runtime
                .assignments
                .facts(target, &job, &jit, &job_ids)
                .await
                .internal()?;
            let _operation = runtime.operation.lock().await;
            let Some(lease) = runtime.store.own_runner_lease(workspace).await.internal()? else {
                return Ok(());
            };
            lease.check().await.internal()?;
            let store = runtime.store.clone();
            let current = tokio::task::spawn_blocking(move || {
                store.apply_own_runner_result(workspace, &expected, OwnRunnerResult::Prepared)
            })
            .await
            .internal()?
            .internal()?;
            if !current {
                return Ok(());
            }
            if matches!(outcome, JobPreparationOutcome::TrustedFacts { .. })
                && !runtime
                    .send_docker_budget(workspace, machine, budget)
                    .await?
            {
                return Ok(());
            }
            let message = ControlMessage::JobPreparation {
                command_id: CommandId(runtime.clock.uuid().to_string()),
                runner_id: RunnerId(runner_id.to_string()),
                job_id: job_ids.worker_job_id,
                outcome,
            };
            let gateway = runtime.gateway.clone();
            match tokio::task::spawn_blocking(move || gateway.send(workspace, machine, message))
                .await
                .internal()?
            {
                Ok(_)
                | Err(ciproxy_gateway::error::Error::NotFound)
                | Err(ciproxy_gateway::error::Error::Unauthorized) => Ok(()),
                Err(error) => Err(error).internal(),
            }
        })
    }
}
