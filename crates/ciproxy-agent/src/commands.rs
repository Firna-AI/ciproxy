//! Authenticated commands. Guest assertions cannot supply job facts.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::Mode;

impl Controller {
    pub(crate) async fn apply(&self, message: ControlMessage) -> Result<Option<AgentMessage>> {
        let (command_id, result, failure) = match message {
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
                        if let Err(error) = self.cache.facts(&runner_id, &facts).await {
                            tracing::warn!(error = %error, "machine cache unavailable; use GitHub cache");
                        }
                        self.vms
                            .send(&vm, &AgentToGuest::JobFacts { job_id, facts })
                            .await
                    }
                    None => Err(Error::RunnerMismatch),
                };
                (command_id, result, CommandFailure::GuestConnection)
            }
            ControlMessage::Drain { command_id } => (
                command_id,
                self.set_mode(Mode::Draining).await,
                CommandFailure::RunnerStop,
            ),
            ControlMessage::Resume { command_id } => {
                let result = if self.state.lock().await.mode == Mode::Removed {
                    Err(Error::Revoked)
                } else {
                    self.set_mode(Mode::Active).await
                };
                (command_id, result, CommandFailure::RunnerStart)
            }
            ControlMessage::Remove { command_id } => (
                command_id,
                self.set_mode(Mode::Removed).await,
                CommandFailure::RunnerStop,
            ),
            ControlMessage::ClearCache { command_id } => (
                command_id,
                self.cache.clear().await,
                CommandFailure::CacheClear,
            ),
            ControlMessage::FetchImage { command_id, image } => {
                let result = self.images.fetch(&image).await;
                let reason = if matches!(result, Err(Error::ImageVerification | Error::ImageLayout))
                {
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

    pub(crate) async fn set_mode(&self, mode: Mode) -> Result<()> {
        let runners = {
            let mut state = self.state.lock().await;
            if state.mode != mode {
                let mut next = state.clone();
                next.mode = mode;
                self.store.save(&next)?;
                *state = next;
            }
            state.runners.clone()
        };
        let mut failure = None;
        if mode != Mode::Active {
            for vm in runners {
                if !matches!(vm.report.state, RunnerState::Running { .. })
                    && let Err(error) = self.request_idle_stop(&vm).await
                {
                    tracing::warn!(error = %error, "idle VM cleanup failed; retry on reconciliation");
                    if failure.is_none() {
                        failure = Some(error);
                    }
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
