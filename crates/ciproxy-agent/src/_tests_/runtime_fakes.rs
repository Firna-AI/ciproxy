//! Pending command and incoming-connection barriers keep daemon tests deterministic.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::reports::MachineReport;
use tokio::sync::{Mutex, mpsc, oneshot};

use crate::boundaries::Connection;
use crate::boundaries::GuestProvenance;
use crate::connection_types::{ConnectionResult, ControlEvent};
use crate::controller::Agent;
use crate::error::Result;

pub(crate) struct WaitingAgent {
    pub(crate) delegate: Arc<dyn Agent>,
    pub(crate) started: mpsc::UnboundedSender<()>,
    pub(crate) barrier: Mutex<Option<oneshot::Receiver<()>>>,
    pub(crate) completed: Arc<AtomicBool>,
}

#[async_trait]
impl Agent for WaitingAgent {
    async fn control_connection(&self, connected: bool) -> Result<()> {
        self.delegate.control_connection(connected).await
    }
    async fn configuration_lost(&self, runner: &str) -> Result<()> {
        self.delegate.configuration_lost(runner).await
    }
    async fn restore(&self) -> Result<()> {
        self.delegate.restore().await
    }
    async fn reconcile(&self) -> Result<()> {
        self.delegate.reconcile().await
    }
    async fn report(&self) -> Result<MachineReport> {
        self.delegate.report().await
    }
    async fn control(&self, message: ControlMessage) -> Result<Option<AgentMessage>> {
        let result = self.delegate.control(message).await;
        self.started.send(()).unwrap();
        let barrier = self.barrier.lock().await.take().unwrap();
        barrier.await.unwrap();
        self.completed.store(true, Ordering::SeqCst);
        result
    }
    async fn guest(
        &self,
        provenance: GuestProvenance,
        runner: &str,
        message: GuestToAgent,
    ) -> Result<Option<AgentMessage>> {
        self.delegate.guest(provenance, runner, message).await
    }
    async fn remove(&self) -> Result<()> {
        self.delegate.remove().await
    }
    async fn finished(&self) -> bool {
        self.delegate.finished().await
    }
}

pub(crate) struct IncomingConnection {
    pub(crate) delegate: Arc<dyn Connection>,
    pub(crate) permit: Mutex<mpsc::UnboundedReceiver<()>>,
}

#[async_trait]
impl Connection for IncomingConnection {
    async fn connect(&self) -> ConnectionResult<()> {
        self.delegate.connect().await
    }
    async fn send(&self, message: &AgentMessage) -> ConnectionResult<()> {
        self.delegate.send(message).await
    }
    async fn receive(&self) -> ConnectionResult<ControlEvent> {
        self.permit.lock().await.recv().await.unwrap();
        self.delegate.receive().await
    }
    async fn receive_handshake(&self) -> ConnectionResult<ControlEvent> {
        self.delegate.receive_handshake().await
    }
}
