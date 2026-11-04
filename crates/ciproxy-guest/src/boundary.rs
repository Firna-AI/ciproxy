//! Side-effect boundaries for supervision, runner processes, and shutdown.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::secret::Secret;

use crate::error::Result;

/// A command read which stays pending while the guest observes its runner.
pub type CommandFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<AgentToGuest>>> + Send + 'a>>;

/// A cancellable wait for a runner observation.
pub type EventFuture<'a> = Pin<Box<dyn Future<Output = Result<RunnerEvent>> + Send + 'a>>;

/// Configuration and observations for the one runner in this guest.
#[cfg_attr(test, unimock::unimock(api = ChannelMock))]
#[async_trait]
pub trait Channel: Send {
    /// Read the next command, or return `None` after a disconnected agent.
    fn receive(&mut self) -> CommandFuture<'_>;
    /// Send an observation without logging its serialized contents.
    async fn send(&mut self, message: GuestToAgent) -> Result<()>;
}

/// A process event which the stock runner actually exposes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunnerEvent {
    /// The listener has connected to GitHub and can take its job.
    Ready,
    /// The runner process has ended.
    Exited(RunnerExit),
}

/// The stock runner process, separate from the guest's control channel.
#[cfg_attr(test, unimock::unimock(api = RunnerMock))]
#[async_trait]
pub trait Runner: Send {
    /// Start one listener with one just-in-time registration.
    async fn start(&mut self, registration: Secret) -> Result<()>;
    /// Wait for readiness or exit while the supervisor handles commands.
    fn next_event(&mut self) -> EventFuture<'_>;
    /// Stop the runner when the agent explicitly sends `Shutdown`.
    async fn stop(&mut self) -> Result<RunnerExit>;
}

/// Power off an own-machine VM, or finish a Boat guest namespace.
#[cfg_attr(test, unimock::unimock(api = PowerMock))]
#[async_trait]
pub trait Power: Send {
    /// End this guest after the runner exits, including an error exit.
    async fn shutdown(&mut self) -> Result<()>;
}
