//! Compact observations preserve exact Worker identities outside the queue value.

use ciproxy_protocol::agent::{AgentMessage, CommandOutcome};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub(crate) enum AgentEvent {
    Runner {
        runner_id: Uuid,
        state: RunnerState,
    },
    CommandResult {
        command_id: Uuid,
        outcome: CommandOutcome,
        /// Image results must match the canonical text of the issued command.
        canonical: bool,
    },
    InvalidRunner,
}

impl AgentEvent {
    pub(crate) fn from_message(message: AgentMessage) -> Option<(Self, Option<JobId>)> {
        let event = match message {
            AgentMessage::RunnerReady { runner_id } => Self::runner(runner_id, RunnerState::Ready),
            AgentMessage::JobStarted { runner_id, job_id } => {
                return Some((Self::runner(runner_id, RunnerState::Busy), Some(job_id)));
            }
            AgentMessage::RunnerExited { runner_id, .. } => {
                Self::runner(runner_id, RunnerState::Stopped)
            }
            AgentMessage::CommandResult {
                command_id,
                outcome,
            } => {
                let id = command_id.0.parse::<Uuid>().ok()?;
                Self::CommandResult {
                    command_id: id,
                    outcome,
                    canonical: id.to_string() == command_id.0,
                }
            }
            _ => return None,
        };
        Some((event, None))
    }

    fn runner(runner_id: RunnerId, state: RunnerState) -> Self {
        match parse_runner_id(&runner_id) {
            Some(id) => Self::Runner {
                runner_id: id,
                state,
            },
            None => Self::InvalidRunner,
        }
    }
}

/// Read a runner identity only in the canonical form sent to the agent.
pub(crate) fn parse_runner_id(runner_id: &RunnerId) -> Option<Uuid> {
    let id = runner_id.0.parse::<Uuid>().ok()?;
    (id.to_string() == runner_id.0).then_some(id)
}
