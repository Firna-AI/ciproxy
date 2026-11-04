//! A report timer, control worker, and guest channels run independently.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::reports::REPORT_INTERVAL_SECONDS;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

use crate::boundaries::{Connection, GuestEvent};
use crate::controller::Agent;
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::Gateway;
use crate::maintenance::Maintenance;
use crate::state::Identity;

#[async_trait]
pub(crate) trait Daemon: Send + Sync {
    async fn run(
        &self,
        identity: Identity,
        guests: mpsc::Receiver<GuestEvent>,
        removing: bool,
    ) -> Result<()>;
}

pub(crate) struct AgentDaemon {
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) gateway: Arc<dyn Gateway>,
    pub(crate) maintenance: Arc<dyn Maintenance>,
}

enum Work {
    Control(Box<ControlMessage>),
    Guest(GuestEvent),
    Reconcile,
}

#[async_trait]
impl Daemon for AgentDaemon {
    async fn run(
        &self,
        identity: Identity,
        mut guests: mpsc::Receiver<GuestEvent>,
        mut removing: bool,
    ) -> Result<()> {
        self.agent.restore().await?;
        let (outbound_tx, mut outbound_rx) = mpsc::channel(128);
        let (control_tx, control_rx) = mpsc::channel(128);
        let worker = tokio::spawn(control_worker(self.agent.clone(), control_rx, outbound_tx));
        let (connected_tx, mut connected_rx) = mpsc::channel(1);
        let mut reconnect = if removing {
            None
        } else {
            Some(start_reconnect(
                self.gateway.clone(),
                identity.clone(),
                connected_tx.clone(),
            ))
        };
        let maintenance = self.maintenance.clone();
        let updater = tokio::spawn(async move {
            maintenance.run().await;
        });
        let mut report_timer = interval(Duration::from_secs(REPORT_INTERVAL_SECONDS));
        report_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut removal_signal =
            signal(SignalKind::user_defined1()).context("listen for local uninstall")?;
        let mut connected = false;
        let mut pending_remove: Option<CommandId> = None;
        loop {
            if pending_remove.is_none() && self.agent.finished().await {
                break;
            }
            let outbound = tokio::select! {
                status = connected_rx.recv(), if !removing && !connected => {
                    reconnect = None;
                    match status {
                        Some(Ok(())) => connected = true,
                        Some(Err(Error::Revoked)) => removing = true,
                        Some(Err(error)) => return Err(error),
                        None => return Err(Error::Disconnected),
                    }
                    None
                }
                message = self.connection.receive(), if connected => {
                    match message {
                        Ok(message) => {
                            if let ControlMessage::Remove { command_id } = &message { pending_remove = Some(command_id.clone()); }
                            control_tx.send(Work::Control(Box::new(message))).await.context("queue control message")?;
                        }
                        Err(error) => {
                            tracing::warn!(error = %error, "control connection failed; preserve VMs");
                            connected = false;
                        }
                    }
                    None
                }
                event = guests.recv() => {
                    match event {
                        Some(event) => { control_tx.send(Work::Guest(event)).await.context("queue guest observation")?; None },
                        None => return Err(Error::Disconnected),
                    }
                }
                message = outbound_rx.recv() => { message }
                _ = report_timer.tick() => {
                    control_tx.send(Work::Reconcile).await.context("queue machine reconciliation")?;
                    if connected { Some(AgentMessage::Report { report: self.agent.report().await? }) } else { None }
                }
                _ = removal_signal.recv() => {
                    self.agent.remove().await?;
                    removing = true;
                    None
                }
            };
            if let Some(AgentMessage::CommandResult { command_id, .. }) = &outbound
                && pending_remove.as_ref() == Some(command_id)
            {
                pending_remove = None;
            }
            if connected
                && let Some(message) = outbound
                && let Err(error) = self.connection.send(&message).await
            {
                tracing::warn!(error = %error, "control send failed; preserve VMs");
                connected = false;
            }
            if !connected && !removing && reconnect.is_none() {
                reconnect = Some(start_reconnect(
                    self.gateway.clone(),
                    identity.clone(),
                    connected_tx.clone(),
                ));
            }
        }
        if let Some(task) = reconnect {
            task.abort();
        }
        worker.abort();
        updater.abort();
        Ok(())
    }
}

fn start_reconnect(
    gateway: Arc<dyn Gateway>,
    identity: Identity,
    result: mpsc::Sender<Result<()>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let _ = result.send(gateway.reconnect(&identity).await).await;
    })
}

async fn control_worker(
    agent: Arc<dyn Agent>,
    mut commands: mpsc::Receiver<Work>,
    outgoing: mpsc::Sender<AgentMessage>,
) {
    while let Some(work) = commands.recv().await {
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
            Work::Guest(event) => agent.guest(&event.runner_id.0, event.message).await,
            Work::Reconcile => agent.reconcile().await.map(|()| None),
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
