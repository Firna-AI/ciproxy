//! One coalesced reconciliation request accompanies serialized control work.

use std::sync::Arc;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use tokio::sync::{Notify, mpsc};

use crate::boundaries::GuestEvent;
use crate::controller::Agent;

pub(crate) enum Work {
    Control(Box<ControlMessage>),
    Guest(GuestEvent),
}

pub(crate) async fn control_worker(
    agent: Arc<dyn Agent>,
    mut commands: mpsc::Receiver<Work>,
    outgoing: mpsc::Sender<AgentMessage>,
    reconciliation: Arc<Notify>,
) {
    loop {
        let work = tokio::select! {
            work = commands.recv() => match work { Some(work) => work, None => return },
            _ = reconciliation.notified() => {
                if let Err(error) = agent.reconcile().await { tracing::warn!(error = %error, "machine reconciliation failed"); }
                continue;
            }
        };
        let result = match work {
            Work::Control(message) if matches!(*message, ControlMessage::FetchImage { .. }) => {
                let agent = agent.clone();
                let outgoing = outgoing.clone();
                tokio::spawn(async move {
                    if let Ok(Some(reply)) = agent.control(*message).await {
                        let _ = outgoing.send(reply).await;
                    }
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
        match result {
            Ok(Some(message)) => {
                if outgoing.send(message).await.is_err() {
                    return;
                }
            }
            Ok(None) => {}
            Err(error) => tracing::warn!(error = %error, "control message failed"),
        }
    }
}
