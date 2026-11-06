//! The Firecracker CONNECT handshake precedes newline-delimited protocol JSON.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::sync::{Mutex, mpsc, watch};
use tokio::time::{MissedTickBehavior, interval, sleep, timeout};

use crate::boundaries::{GuestEvent, GuestProvenance};
use crate::error::{Error, ErrorResultExt, Result};
use crate::guest_transport::GuestTransport;
use crate::layout::unit_name;
use crate::process::{Commands, UnitState};
use crate::state::VmRecord;
use crate::vsock::Configuration;

pub(crate) struct GuestBridge {
    pub(crate) configuration: Arc<Mutex<Configuration>>,
    pub(crate) transport: Arc<dyn GuestTransport>,
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) events: mpsc::Sender<GuestEvent>,
    pub(crate) vm: VmRecord,
    pub(crate) initial: AtomicBool,
}

impl GuestBridge {
    pub(crate) async fn run(
        self,
        mut receiver: mpsc::Receiver<AgentToGuest>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        loop {
            match self.commands.unit(&unit_name(&self.vm)).await {
                Ok(UnitState::Exited(code)) => {
                    let _ = self
                        .emit(
                            GuestToAgent::RunnerExited {
                                runner_id: self.vm.report.runner_id.clone(),
                                exit: RunnerExit { code: Some(code) },
                            },
                            GuestProvenance::Recovery,
                        )
                        .await;
                    return;
                }
                Ok(UnitState::Absent) => {
                    let _ = self
                        .emit(
                            GuestToAgent::RunnerExited {
                                runner_id: self.vm.report.runner_id.clone(),
                                exit: RunnerExit { code: None },
                            },
                            GuestProvenance::Recovery,
                        )
                        .await;
                    return;
                }
                Ok(UnitState::Active) => {}
                Err(error) => tracing::warn!(error = %error, "cannot inspect VM; preserve it"),
            }
            if let Err(error) = self.session(&mut receiver, &mut shutdown).await {
                tracing::debug!(error = %error, "guest channel unavailable; preserve VM and reconnect");
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn session(
        &self,
        receiver: &mut mpsc::Receiver<AgentToGuest>,
        shutdown: &mut watch::Receiver<bool>,
    ) -> Result<()> {
        let mut socket = self.transport.connect(&self.vm).await?;
        let message = timeout(Duration::from_secs(30), socket.receive())
            .await
            .context("wait for guest hello")??;
        let GuestToAgent::Hello {
            protocol_version,
            image_id,
            configuration_received,
            handoff_protocol_version,
        } = &message
        else {
            return Err(Error::GuestMismatch);
        };
        if *protocol_version != ProtocolVersion::CURRENT
            || *handoff_protocol_version != ciproxy_protocol::guest::HANDOFF_PROTOCOL_VERSION
            || *image_id != self.vm.report.image_id
        {
            return Err(Error::GuestMismatch);
        }
        let configure = {
            let mut pending = self.configuration.lock().await;
            pending.received = Some(*configuration_received);
            if *configuration_received {
                pending.pending = None;
            }
            pending.pending.clone()
        };
        let mut configuration_lost = !configuration_received && configure.is_none();
        let hello = message.clone();
        self.events
            .send(GuestEvent {
                provenance: GuestProvenance::Recovery,
                runner_id: self.vm.report.runner_id.clone(),
                configuration_lost: !configuration_received && configure.is_none(),
                message,
            })
            .await
            .context("deliver guest hello")?;
        let mut configuration_sent = false;
        let mut initial = false;
        if let Some(configure) = configure.as_ref() {
            initial = self.initial.swap(false, Ordering::AcqRel);
            socket.send(configure).await?;
            configuration_sent = true;
        }
        let stop_requested = *shutdown.borrow_and_update();
        if stop_requested {
            socket.send(&AgentToGuest::Shutdown).await?;
        }
        let mut health = interval(Duration::from_secs(10));
        health.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                message = socket.receive() => {
                    let message = message?;
                    if let GuestToAgent::RunnerConfigured { runner_id } = &message {
                        if *runner_id != self.vm.report.runner_id { return Err(Error::GuestMismatch); }
                        { let mut current = self.configuration.lock().await; current.pending = None; current.received = Some(true); }
                        configuration_lost = false;
                    }
                    let provenance = if initial && configuration_sent {
                        GuestProvenance::InitialConfiguration
                    } else {
                        GuestProvenance::Recovery
                    };
                    self.emit(message, provenance).await?;
                }
                message = receiver.recv() => {
                    let message = message.ok_or(Error::Disconnected)?;
                    if matches!(message, AgentToGuest::ConfigureRunner { .. }) {
                        initial |= self.initial.swap(false, Ordering::AcqRel);
                    }
                    socket.send(&message).await?;
                    if matches!(message, AgentToGuest::ConfigureRunner { .. }) {
                        configuration_sent = true;
                    }
                }
                changed = shutdown.changed() => {
                    changed.context("read pending guest shutdown")?;
                    socket.send(&AgentToGuest::Shutdown).await?;
                }
                _ = health.tick() => {
                    if configuration_lost && self.configuration.lock().await.lost() {
                        self.events.send(GuestEvent {
                            provenance: GuestProvenance::Recovery,
                            runner_id: self.vm.report.runner_id.clone(),
                            configuration_lost: true,
                            message: hello.clone(),
                        }).await.context("replay lost configuration observation")?;
                    }
                    if self.commands.unit(&unit_name(&self.vm)).await? != UnitState::Active {
                        return Err(Error::Disconnected);
                    }
                }
            }
        }
    }

    async fn emit(&self, message: GuestToAgent, provenance: GuestProvenance) -> Result<()> {
        self.events
            .send(GuestEvent {
                provenance,
                runner_id: self.vm.report.runner_id.clone(),
                configuration_lost: false,
                message,
            })
            .await
            .context("deliver guest event")?;
        Ok(())
    }
}
