//! Prepare checked local resources only after the host owns trusted job facts.

use ciproxy_protocol::guest::{AgentToGuest, JobDecision};
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::runners::RunnerState;

use crate::cache_authority::CacheAuthority;
use crate::capacity::machine_resources;
use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::VmRecord;

impl Controller {
    pub(crate) async fn prepare_job(&self, vm: &VmRecord, facts: JobFacts) -> Result<()> {
        let RunnerState::Running { job_id } = &vm.report.state else {
            return Err(Error::GuestMismatch);
        };
        let facts = self.trusted_facts(vm, facts).await?;
        let first_preparation = self.begin_preparation(vm).await?;
        let allowed = self.state.lock().await.runners.iter().any(|record| {
            record.local_id == vm.local_id && record.cache_authority != CacheAuthority::GitHubOnly
        });
        let prepared = if allowed {
            self.cache.prepare(&vm.report.runner_id, &facts).await
        } else {
            Ok(None)
        };
        let (cache, scope, data_bytes) = match prepared {
            Ok(Some((cache, scope, bytes))) => (Some(cache), Some(scope), bytes),
            Ok(None) => (None, None, 0),
            Err(error) => {
                tracing::warn!(%error, "cache session unavailable; use GitHub cache");
                (None, None, 0)
            }
        };
        match cache {
            None => self.release_github(vm).await,
            Some(cache) => {
                let budget = self.state.lock().await.docker_budget.clone();
                let spare = match budget {
                    Some(budget)
                        if first_preparation
                            && self.dockers.is_some()
                            && budget.image.image_id == vm.report.image_id
                            && self.images.report()?.selected() == Some(&budget.image) =>
                    {
                        budget.spare_bytes
                    }
                    _ => 0,
                };
                let free_pool = if first_preparation && self.dockers.is_some() {
                    let hardware = self.host.hardware()?;
                    let state = self.state.lock().await;
                    machine_resources(&hardware, self.image_budget)
                        .available(state.runners.iter().map(|vm| vm.report.size))
                        .job_disk_bytes
                } else {
                    0
                };
                let docker = match (&self.dockers, scope, first_preparation) {
                    (Some(dockers), Some(scope), true) => {
                        match dockers
                            .prepare(vm, &facts, &scope, data_bytes, spare, free_pool)
                            .await
                        {
                            Ok(Some(disk)) => match self.vms.swap_docker(vm).await {
                                Ok(()) => Some(disk),
                                Err(error) => {
                                    tracing::warn!(%error, "Docker drive unavailable; use root disk");
                                    None
                                }
                            },
                            Ok(None) => None,
                            Err(error) => {
                                tracing::warn!(%error, "Docker cache unavailable; use root disk");
                                None
                            }
                        }
                    }
                    _ => None,
                };
                self.vms
                    .send(
                        vm,
                        &AgentToGuest::PrepareJob {
                            job_id: job_id.clone(),
                            decision: JobDecision::MachineCache {
                                facts: Box::new(facts.clone()),
                                cache,
                                docker,
                            },
                        },
                    )
                    .await
            }
        }
    }
}
