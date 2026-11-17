//! Bounded, ordered lifecycle handoff from synchronous gateway workers.

use std::sync::Arc;

use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::AgentMessage;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};
use crate::runtime::{MachineRuntime, OwnMachines};
use crate::runtime_tasks::RuntimeTasks;
use crate::tasks::OwnedTask;

pub(crate) enum Event {
    Agent(Session, Box<AgentMessage>),
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
            let task = self.handle.spawn(async move {
                while let Some(event) = receiver.recv().await {
                    let worker = runtime.worker();
                    let mut event_task = OwnedTask(tokio::spawn(async move { consume(&worker, event).await }));
                    match (&mut event_task.0).await {
                        Ok(Ok(())) => (),
                        Ok(Err(error)) => tracing::warn!(%error,"agent event failed; full reports and saved stops retain recovery"),
                        Err(error) => tracing::warn!(%error,"agent event task failed; ordered consumption continues"),
                    }
                    let Some(tasks) = tasks.upgrade() else { return; };
                    tasks.queue_finished();
                }
            });
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
            let store = runtime.store.clone();
            let current = tokio::task::spawn_blocking(move || {
                store.machine_session(session.workspace_id, session.machine_id)
            })
            .await
            .internal()?
            .internal()?;
            if current == Some(session.id) {
                runtime
                    .agent(session.workspace_id, session.machine_id, *message)
                    .await?;
            }
            Ok(())
        }
        Event::Removed(workspace, machine) => runtime.machine_removed(workspace, machine).await,
    }
}
