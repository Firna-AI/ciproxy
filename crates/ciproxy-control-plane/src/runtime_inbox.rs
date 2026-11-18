//! Bounded, ordered lifecycle handoff from synchronous gateway workers.

use std::sync::Arc;

use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::{AgentMessage, CommandOutcome};
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_store_interface::runners::RunnerState;
use tokio::sync::mpsc;
use tracing::Instrument;
use tracing::instrument::WithSubscriber;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::{MachineRuntime, OwnMachines};
use crate::runtime_tasks::RuntimeTasks;
use crate::tasks::OwnedTask;

/// Bounded runtime observations with no retained wire strings or reports.
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
    pub(crate) fn from_message(message: AgentMessage) -> Option<Self> {
        match message {
            AgentMessage::RunnerReady { runner_id } => {
                Some(Self::runner(runner_id, RunnerState::Ready))
            }
            AgentMessage::JobStarted { runner_id, .. } => {
                Some(Self::runner(runner_id, RunnerState::Busy))
            }
            AgentMessage::RunnerExited { runner_id, .. } => {
                Some(Self::runner(runner_id, RunnerState::Stopped))
            }
            AgentMessage::CommandResult {
                command_id,
                outcome,
            } => {
                let id = command_id.0.parse::<Uuid>().ok()?;
                Some(Self::CommandResult {
                    command_id: id,
                    outcome,
                    canonical: id.to_string() == command_id.0,
                })
            }
            _ => None,
        }
    }

    fn runner(runner_id: RunnerId, state: RunnerState) -> Self {
        match runner_id.0.parse::<Uuid>() {
            Ok(id) if id.to_string() == runner_id.0 => Self::Runner {
                runner_id: id,
                state,
            },
            _ => Self::InvalidRunner,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Event {
    Agent(Session, AgentEvent),
    Removed(Uuid, Uuid),
}

impl RuntimeTasks {
    pub(crate) fn enqueue(self: &Arc<Self>, runtime: MachineRuntime, event: Event) {
        if !self.queue_added() {
            return;
        }
        let sender = self.inbox.get_or_init(|| {
            let (sender, mut receiver) = mpsc::channel(1024);
            let tasks = Arc::downgrade(self);
            let task = self.handle.spawn(
                async move {
                    while let Some(event) = receiver.recv().await {
                        let worker = runtime.worker();
                        let mut event_task = OwnedTask(tokio::spawn(
                            async move { consume(&worker, event).await }.with_current_subscriber(),
                        ));
                        match (&mut event_task.0).await {
                            Ok(Ok(())) => (),
                            Ok(Err(error)) => tracing::warn!(%error,"agent event failed; full reports and saved stops retain recovery"),
                            Err(error) => tracing::warn!(%error,"agent event task failed; ordered consumption continues"),
                        }
                        let Some(tasks) = tasks.upgrade() else { return; };
                        tasks.queue_finished();
                    }
                }
                .with_current_subscriber(),
            );
            *self.inbox_owner.lock().unwrap_or_else(|error| error.into_inner()) = Some(OwnedTask(task));
            sender
        });
        if let Err(error) = sender.try_send(event) {
            self.queue_finished();
            tracing::warn!(
                full = matches!(error, mpsc::error::TrySendError::Full(_)),
                "agent event queue unavailable; full reports and saved stops retain recovery"
            );
        }
    }
}

async fn consume(runtime: &MachineRuntime, event: Event) -> Result<()> {
    match event {
        Event::Agent(session, message) => {
            let _operation = runtime
                .operation
                .lock()
                .instrument(tracing::trace_span!("agent_event_operation"))
                .await;
            let store = runtime.store.clone();
            let current = tokio::task::spawn_blocking(move || {
                store.machine_session(session.workspace_id, session.machine_id)
            })
            .await
            .internal()?
            .internal()?;
            if current == Some(session.id) {
                runtime
                    .agent_event(session.workspace_id, session.machine_id, message)
                    .await?;
            }
            Ok(())
        }
        Event::Removed(workspace, machine) => runtime.machine_removed(workspace, machine).await,
    }
}
