//! One coalesced reconciliation request accompanies serialized control work.

use std::sync::Arc;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use tokio::sync::{Notify, mpsc};

use crate::boundaries::GuestEvent;
use crate::controller::Agent;
use crate::error::Result;

pub(crate) enum Work {
    Control(Box<ControlMessage>),
    Guest(GuestEvent),
    Reconcile,
}

/// Every finished operation wakes the daemon, including errors and no-reply work.
pub(crate) struct Completion {
    /// The operation's optional wire reply or local failure.
    pub(crate) result: Result<Option<AgentMessage>>,
}

pub(crate) async fn control_worker(
    agent: Arc<dyn Agent>,
    mut commands: mpsc::Receiver<Work>,
    completions: mpsc::Sender<Completion>,
    reconciliation: Arc<Notify>,
) {
    loop {
        let work = tokio::select! {
            work = commands.recv() => match work { Some(work) => work, None => return },
            _ = reconciliation.notified() => Work::Reconcile,
        };
        let result = match work {
            Work::Reconcile => agent.reconcile().await.map(|_| None),
            Work::Control(message) if matches!(*message, ControlMessage::FetchImage { .. }) => {
                let agent = agent.clone();
                let completions = completions.clone();
                tokio::spawn(async move {
                    let result = agent.control(*message).await;
                    let _ = completions.send(Completion { result }).await;
                });
                continue;
            }
            Work::Control(message) => agent.control(*message).await,
            Work::Guest(event) => {
                let recorded = if matches!(event.message, GuestToAgent::Hello { .. }) {
                    agent.configuration_lost(&event.runner_id.0).await
                } else {
                    Ok(())
                };
                match recorded {
                    Ok(()) => {
                        agent
                            .guest(event.provenance, &event.runner_id.0, event.message)
                            .await
                    }
                    Err(error) => Err(error),
                }
            }
        };
        if completions.send(Completion { result }).await.is_err() {
            return;
        }
    }
}
