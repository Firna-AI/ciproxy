//! Authenticated commands. Guest assertions cannot supply job facts.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::mode::ModeAction;

impl Controller {
    pub(crate) async fn apply(&self, message: ControlMessage) -> Result<Option<AgentMessage>> {
        let (command_id, result, failure) = match message {
            ControlMessage::DockerBudget { command_id, budget } => {
                let mut state = self.state.lock().await;
                let mut next = state.clone();
                next.docker_budget = budget;
                let result = if *state == next {
                    Ok(())
                } else {
                    self.store.save(&next)
                };
                if result.is_ok() {
                    *state = next;
                } else {
                    state.docker_budget = None;
                }
                (command_id, result, CommandFailure::RunnerStart)
            }
            ControlMessage::StartRunner { command_id, runner } => (
                command_id,
                self.start(runner).await,
                CommandFailure::RunnerStart,
            ),
            ControlMessage::StopRunner {
                command_id,
                organization_id,
                runner_id,
            } => {
                let vm = self
                    .state
                    .lock()
                    .await
                    .runners
                    .iter()
                    .find(|vm| {
                        vm.report.runner_id == runner_id
                            && vm.report.organization_id == organization_id
                            && !matches!(vm.report.state, RunnerState::Running { .. })
                    })
                    .cloned();
                let result = match vm {
                    Some(vm) => self.request_idle_stop(&vm).await,
                    None => Err(Error::RunnerMismatch),
                };
                (command_id, result, CommandFailure::RunnerStop)
            }
            ControlMessage::JobFacts {
                command_id,
                runner_id,
                job_id,
                facts,
            } => {
                let vm = self
                    .state
                    .lock()
                    .await
                    .runners
                    .iter()
                    .find(|vm| {
                        vm.report.runner_id == runner_id
                            && vm.report.organization_id == facts.organization_id
                            && vm.report.state
                                == RunnerState::Running {
                                    job_id: job_id.clone(),
                                }
                    })
                    .cloned();
                let result = match vm {
                    Some(vm) => {
                        let facts = self.trusted_facts(&vm, facts).await?;
                        let first_preparation = self.begin_preparation(&vm).await?;
                        let prepared = self.cache.prepare(&runner_id, &facts).await;
                        let (cache, scope, data_bytes) = match prepared {
                            Ok(Some((cache, scope, bytes))) => (Some(cache), Some(scope), bytes),
                            Ok(None) => (None, None, 0),
                            Err(error) => {
                                tracing::warn!(%error, "cache session unavailable; use GitHub cache");
                                (None, None, 0)
                            }
                        };
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
                            crate::capacity::machine_resources(&hardware, self.image_budget)
                                .available(state.runners.iter().map(|vm| vm.report.size))
                                .job_disk_bytes
                        } else {
                            0
                        };
                        let docker = match (&self.dockers, scope, first_preparation) {
                            (Some(dockers), Some(scope), true) => {
                                match dockers
                                    .prepare(&vm, &facts, &scope, data_bytes, spare, free_pool)
                                    .await
                                {
                                    Ok(Some(disk)) => match self.vms.swap_docker(&vm).await {
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
                                &vm,
                                &AgentToGuest::PrepareJob {
                                    job_id: job_id.clone(),
                                    facts: facts.clone(),
                                    cache,
                                    docker,
                                },
                            )
                            .await
                    }
                    None => Err(Error::RunnerMismatch),
                };
                (command_id, result, CommandFailure::GuestConnection)
            }
            ControlMessage::Drain { command_id } => (
                command_id,
                self.change_mode(ModeAction::Drain).await,
                CommandFailure::RunnerStop,
            ),
            ControlMessage::Resume { command_id } => (
                command_id,
                self.change_mode(ModeAction::Resume).await,
                CommandFailure::RunnerStart,
            ),
            ControlMessage::Remove { command_id } => (
                command_id,
                self.change_mode(ModeAction::Remove).await,
                CommandFailure::RunnerStop,
            ),
            ControlMessage::ClearCache { command_id } => (
                command_id,
                self.cache.clear().await,
                CommandFailure::CacheClear,
            ),
            ControlMessage::FetchImage { command_id, image } => {
                let result = self.images.fetch(&image).await;
                let reason = if matches!(result, Err(Error::ImageVerification)) {
                    CommandFailure::ImageVerification
                } else {
                    CommandFailure::ImageDownload
                };
                (command_id, result, reason)
            }
            ControlMessage::Welcome { protocol_version } => {
                if protocol_version != ProtocolVersion::CURRENT {
                    return Err(Error::ProtocolVersion);
                }
                return Ok(None);
            }
            ControlMessage::Registered { .. } => return Err(Error::IdentityMismatch),
        };
        let outcome = match result {
            Ok(()) => CommandOutcome::Applied,
            Err(error) => {
                tracing::warn!(error = %error, "control command failed");
                CommandOutcome::Failed { reason: failure }
            }
        };
        Ok(Some(AgentMessage::CommandResult {
            command_id,
            outcome,
        }))
    }
}
