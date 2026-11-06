//! Supervise one runner without tying its lifetime to the agent connection.

use async_trait::async_trait;
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::{ImageId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::version::ProtocolVersion;
use tracing::warn;

use crate::boundary::{Channel, Power, Runner, RunnerEvent};
use crate::error::Result;

/// One fresh VM or Boat namespace lifecycle.
#[async_trait]
pub trait Guest: Send {
    /// Run at most one runner and finish the guest on every local exit.
    async fn run(&mut self) -> Result<()>;
}

/// Guest supervisor with injected side-effect boundaries.
pub struct Supervisor {
    /// Identity of the installed image, never a credential.
    pub image_id: ImageId,
    /// Agent vsock stream or Boat configuration and exit callback.
    pub channel: Box<dyn Channel>,
    /// One stock GitHub runner process.
    pub runner: Box<dyn Runner>,
    /// Shutdown appropriate for this image's execution mode.
    pub power: Box<dyn Power>,
}

#[async_trait]
impl Guest for Supervisor {
    async fn run(&mut self) -> Result<()> {
        let result = self.run_one().await;
        let shutdown = self.power.shutdown().await;
        result?;
        shutdown
    }
}

impl Supervisor {
    async fn report(&mut self, message: GuestToAgent) {
        if self.channel.send(message).await.is_err() {
            warn!("guest observation could not be delivered");
        }
    }

    async fn run_one(&mut self) -> Result<()> {
        self.report(GuestToAgent::Hello {
            protocol_version: ProtocolVersion::CURRENT,
            image_id: self.image_id.clone(),
            configuration_received: false,
        })
        .await;
        loop {
            match self.channel.receive().await? {
                Some(AgentToGuest::ConfigureRunner {
                    runner_id,
                    registration,
                    ..
                }) => {
                    self.report(GuestToAgent::RunnerConfigured {
                        runner_id: runner_id.clone(),
                    })
                    .await;
                    if let Err(error) = self.runner.start(registration).await {
                        self.report_exit(&runner_id, RunnerExit { code: None })
                            .await;
                        return Err(error);
                    }
                    let result = self.supervise(&runner_id).await;
                    if result.is_err() {
                        let exit = self.runner.stop().await;
                        if let Ok(exit) = exit {
                            self.report_exit(&runner_id, exit).await;
                        }
                    }
                    return result;
                }
                Some(AgentToGuest::Shutdown) | None => return Ok(()),
                Some(_) => warn!("runner configuration has not arrived"),
            }
        }
    }

    async fn report_exit(&mut self, runner_id: &RunnerId, exit: RunnerExit) {
        self.report(GuestToAgent::RunnerExited {
            runner_id: runner_id.clone(),
            exit,
        })
        .await;
    }

    async fn supervise(&mut self, runner_id: &RunnerId) -> Result<()> {
        let mut connected = true;
        let mut running = None;
        loop {
            tokio::select! {
                biased;
                command = self.channel.receive(), if connected => match command {
                    Ok(Some(AgentToGuest::Shutdown)) => {
                        if running.is_some() { continue; }
                        let exit = self.runner.stop().await?;
                        self.report_exit(runner_id, exit).await;
                        return Ok(());
                    }
                    Ok(None) | Err(_) => {
                        connected = false;
                        warn!("agent disconnected; runner continues");
                    }
                    Ok(Some(AgentToGuest::ConfigureRunner { runner_id: configured, .. })) => {
                        if configured == *runner_id {
                            self.report(GuestToAgent::RunnerConfigured { runner_id: runner_id.clone() }).await;
                        }
                    }
                    Ok(Some(AgentToGuest::PrepareJob { job_id, facts, cache, docker })) if running.as_ref() == Some(&job_id) => {
                        match self.runner.prepare(job_id.clone(), facts, cache, docker).await {
                            Ok(outcome) => self.report(GuestToAgent::JobPrepared { runner_id: runner_id.clone(), job_id,outcome }).await,
                            Err(error) => {warn!(%error, "local preparation unavailable; runner keeps its GitHub fallback"); self.report(GuestToAgent::JobPrepared{runner_id:runner_id.clone(),job_id,outcome:ciproxy_protocol::guest::JobPreparation::default()}).await;},
                        }
                    }
                    Ok(Some(_)) => warn!("cache integration is not installed"),
                },
                event = self.runner.next_event() => match event? {
                    RunnerEvent::Starting { job_id, runtime } => {
                        running = Some(job_id.clone());
                        self.report(GuestToAgent::JobStarting { runner_id: runner_id.clone(), job_id, runtime }).await;
                    }
                    RunnerEvent::Ready => { if running.is_none() { self.report(GuestToAgent::RunnerReady {
                        runner_id: runner_id.clone(),
                    }).await; } },
                    RunnerEvent::Exited(exit) => {
                        self.report_exit(runner_id, exit).await;
                        return Ok(());
                    }
                },
            }
        }
    }
}

#[cfg(test)]
#[path = "_tests_/service_tests.rs"]
mod service_tests;

#[cfg(test)]
#[path = "_tests_/preparation_tests.rs"]
mod preparation_tests;
