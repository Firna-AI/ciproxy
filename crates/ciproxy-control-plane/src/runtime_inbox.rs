//! Bounded, ordered lifecycle handoff from synchronous gateway workers.

use std::sync::Arc;

use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::JobId;
use tokio::sync::mpsc;
use tracing::Instrument;
use tracing::instrument::WithSubscriber;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::{MachineRuntime, OwnMachines};
use crate::runtime_agent_event::AgentEvent;
use crate::runtime_payloads::{PayloadId, WorkerPayloads};
use crate::runtime_tasks::RuntimeTasks;
use crate::tasks::OwnedTask;

/// Queue handles own no variable-size data.
#[derive(Clone, Copy)]
pub(crate) enum Event {
    Agent(Session, AgentEvent, Option<PayloadId>),
    Removed(Uuid, Uuid),
}

impl RuntimeTasks {
    pub(crate) fn enqueue_agent(
        self: &Arc<Self>,
        runtime: MachineRuntime,
        session: Session,
        message: AgentMessage,
    ) {
        if let Some((message, worker)) = AgentEvent::from_message(message) {
            self.enqueue(runtime, Event::Agent(session, message, None), worker);
        }
    }

    pub(crate) fn enqueue_removed(
        self: &Arc<Self>,
        runtime: MachineRuntime,
        workspace: Uuid,
        machine: Uuid,
    ) {
        self.enqueue(runtime, Event::Removed(workspace, machine), None);
    }

    fn enqueue(self: &Arc<Self>, runtime: MachineRuntime, event: Event, worker: Option<JobId>) {
        let Some(mut admission) = self.payloads.admission() else {
            tracing::warn!(
                "agent event queue stopped; full reports and saved stops retain recovery"
            );
            return;
        };
        if !self.queue_added() {
            tracing::warn!(
                "agent event queue stopped; full reports and saved stops retain recovery"
            );
            return;
        }
        let sender = self.inbox.get_or_init(|| {
            let (sender, mut receiver) = mpsc::channel(1024);
            let tasks = Arc::downgrade(self);
            let payloads = self.payloads.clone();
            let task = self.handle.spawn(
                async move {
                    while let Some(event) = receiver.recv().await {
                        let worker = runtime.worker();
                        let payloads = payloads.clone();
                        let mut event_task = OwnedTask(tokio::spawn(
                            async move { consume(&worker, event, &payloads).await }
                                .with_current_subscriber(),
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
        let permit = match sender.try_reserve() {
            Ok(permit) => permit,
            Err(error) => {
                self.queue_finished();
                tracing::warn!(
                    full = matches!(error, mpsc::error::TrySendError::Full(_)),
                    "agent event queue unavailable; full reports and saved stops retain recovery"
                );
                return;
            }
        };
        let event = match (event, worker) {
            (Event::Agent(session, message, _), Some(worker)) => {
                let Some(payload) = admission.insert(worker) else {
                    self.queue_finished();
                    tracing::warn!(
                        name: "agent_event_payload_dropped",
                        "agent event payload budget full; the next full report retains recovery"
                    );
                    return;
                };
                Event::Agent(session, message, Some(payload))
            }
            (event, _) => event,
        };
        permit.send(event);
    }
}

async fn consume(runtime: &MachineRuntime, event: Event, payloads: &WorkerPayloads) -> Result<()> {
    match event {
        Event::Agent(session, message, payload) => {
            let worker = match payload {
                Some(id) => {
                    let Some(worker) = payloads.take(id) else {
                        return Ok(());
                    };
                    Some(worker)
                }
                None => None,
            };
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
                    .agent_event(session.workspace_id, session.machine_id, message, worker)
                    .await?;
            }
            Ok(())
        }
        Event::Removed(workspace, machine) => runtime.machine_removed(workspace, machine).await,
    }
}
