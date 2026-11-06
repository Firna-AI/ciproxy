//! Authenticated commands. Guest assertions cannot supply job facts.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::mode::ModeAction;
use crate::state::PreparationState;

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
                    })
                    .cloned();
                let result = match vm {
                    Some(vm) if matches!(vm.report.state, RunnerState::Running { .. }) => {
                        if vm.job_facts.is_none() && vm.preparation == PreparationState::Waiting {
                            self.release_github(&vm).await?;
                        }
                        Err(Error::RunnerMismatch)
                    }
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
                    Some(vm) => self.prepare_job(&vm, facts).await,
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
