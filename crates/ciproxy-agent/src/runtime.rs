//! Independent report delivery keeps working while command work is pending.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use tokio::sync::{Notify, mpsc, watch};
use tokio::task::JoinSet;

use crate::boundaries::{Connection, Delay, GuestEvent};
use crate::connection_types::ControlEvent;
use crate::controller::Agent;
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::Gateway;
use crate::maintenance::Maintenance;
use crate::runtime_reports::reporting;
use crate::runtime_worker::{Completion, Work, control_worker};
use crate::state::Identity;

#[async_trait]
pub(crate) trait Daemon: Send + Sync {
    async fn run(
        &self,
        identity: Option<Identity>,
        guests: mpsc::Receiver<GuestEvent>,
        removing: bool,
        removal: mpsc::Receiver<()>,
    ) -> Result<()>;
}

pub(crate) struct AgentDaemon {
    pub(crate) agent: Arc<dyn Agent>,
    pub(crate) connection: Arc<dyn Connection>,
    pub(crate) gateway: Arc<dyn Gateway>,
    pub(crate) maintenance: Arc<dyn Maintenance>,
    pub(crate) report_delay: Arc<dyn Delay>,
}

struct LoopChannels {
    completions: mpsc::Receiver<Completion>,
    work: mpsc::Sender<Work>,
    connected: watch::Sender<bool>,
}

#[async_trait]
impl Daemon for AgentDaemon {
    async fn run(
        &self,
        identity: Option<Identity>,
        guests: mpsc::Receiver<GuestEvent>,
        removing: bool,
        removal: mpsc::Receiver<()>,
    ) -> Result<()> {
        if !removing && identity.is_none() {
            return Err(Error::NotInstalled);
        }
        self.agent.restore().await?;
        self.agent.control_connection(false).await?;
        let (completions, receive) = mpsc::channel(128);
        let (work, commands) = mpsc::channel(128);
        let reconciliation = Arc::new(Notify::new());
        let (connected, _) = watch::channel(false);
        let mut tasks = JoinSet::new();
        tasks.spawn(control_worker(
            self.agent.clone(),
            commands,
            completions,
            reconciliation.clone(),
        ));
        let maintenance = self.maintenance.clone();
        tasks.spawn(async move {
            maintenance.run().await;
        });
        let reports = reporting(
            self.agent.clone(),
            self.connection.clone(),
            self.report_delay.clone(),
            connected.clone(),
            reconciliation,
        );
        let input = LoopChannels {
            completions: receive,
            work,
            connected,
        };
        let result = tokio::select! {
            result = self.run_loop(identity, guests, removing, removal, input) => result,
            result = reports => result,
        };
        tasks.shutdown().await;
        result
    }
}

impl AgentDaemon {
    async fn run_loop(
        &self,
        identity: Option<Identity>,
        mut guests: mpsc::Receiver<GuestEvent>,
        mut removing: bool,
        mut removal: mpsc::Receiver<()>,
        mut channels: LoopChannels,
    ) -> Result<()> {
        let mut reconnections = JoinSet::new();
        let (reconnected, mut status) = mpsc::channel(1);
        let mut connection_state = channels.connected.subscribe();
        let mut connecting = false;
        let mut removal_open = true;
        let mut pending_work = None;
        let mut completion_requested = true;
        let mut terminal_complete = false;
        let mut available = false;
        let mut pending_remove: Option<CommandId> = None;
        let result = loop {
            if pending_remove.is_none() && terminal_complete {
                break Ok(());
            }
            if pending_work.is_none() && completion_requested {
                pending_work = Some(Work::Complete);
                completion_requested = false;
            }
            let connected = *connection_state.borrow_and_update();
            if connected != available {
                self.agent.control_connection(connected).await?;
                available = connected;
            }
            if !connected && !removing && !connecting {
                let gateway = self.gateway.clone();
                let identity = identity.clone().ok_or(Error::NotInstalled)?;
                let result = reconnected.clone();
                reconnections.spawn(async move {
                    let _ = result.send(gateway.reconnect(&identity).await).await;
                });
                connecting = true;
            }
            let outbound = tokio::select! {
                result = status.recv(), if connecting => {
                    connecting = false;
                    match result {
                        Some(Ok(())) => { channels.connected.send_replace(true); }
                        Some(Err(Error::Revoked)) => { removing = true; completion_requested = true; channels.connected.send_replace(false); }
                        Some(Err(error)) => break Err(error),
                        None => break Err(Error::Disconnected),
                    }
                    None
                }
                message = self.connection.receive(), if connected && pending_work.is_none() => {
                    match message {
                        Ok(ControlEvent::Message(message)) => {
                            if let ControlMessage::Remove { command_id } = &*message { pending_remove = Some(command_id.clone()); }
                            pending_work = Some(Work::Control(message));
                        }
                        Ok(ControlEvent::CredentialRejected) => {
                            self.agent.remove().await?;
                            removing = true;
                            completion_requested = true;
                            channels.connected.send_replace(false);
                        }
                        Err(error) => {
                            tracing::warn!(error = %error, "control connection failed; preserve VMs");
                            channels.connected.send_replace(false);
                        }
                    }
                    None
                }
                event = guests.recv(), if pending_work.is_none() => {
                    match event { Some(event) => pending_work = Some(Work::Guest(event)), None => break Err(Error::Disconnected) }
                    None
                }
                permit = channels.work.reserve(), if pending_work.is_some() => {
                    let permit = permit.context("reserve control queue capacity")?;
                    if let Some(work) = pending_work.take() { permit.send(work); }
                    None
                }
                completion = channels.completions.recv() => {
                    match completion {
                        Some(Completion::Work(Ok(reply))) => reply.map(|reply| *reply),
                        Some(Completion::Terminal(Ok(complete))) => {
                            terminal_complete = complete;
                            None
                        }
                        Some(Completion::Work(Err(error)) | Completion::Terminal(Err(error))) => {
                            tracing::warn!(%error, "agent work failed");
                            None
                        }
                        None => break Err(Error::Disconnected),
                    }
                }
                event = removal.recv(), if removal_open => {
                    match event { Some(()) => { self.agent.remove().await?; removing = true; completion_requested = true; }, None => removal_open = false }
                    None
                }
                _ = connection_state.changed() => { None }
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
                channels.connected.send_replace(false);
            }
        };
        reconnections.shutdown().await;
        result
    }
}

#[cfg(test)]
#[path = "_tests_/runtime_tests.rs"]
mod runtime_tests;

#[cfg(test)]
#[path = "_tests_/runtime_removal_tests.rs"]
mod runtime_removal_tests;

#[cfg(test)]
#[path = "_tests_/terminal_runtime_tests.rs"]
mod terminal_runtime_tests;
