//! Report scheduling and delivery do not depend on the command worker queue.

use std::sync::Arc;

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::reports::REPORT_INTERVAL_SECONDS;
use futures_util::future::try_join;
use tokio::sync::{Notify, watch};

use crate::boundaries::{Connection, Delay};
use crate::controller::Agent;
use crate::error::Result;

pub(crate) async fn reporting(
    agent: Arc<dyn Agent>,
    connection: Arc<dyn Connection>,
    delay: Arc<dyn Delay>,
    connected: watch::Sender<bool>,
    reconciliation: Arc<Notify>,
) -> Result<()> {
    let (ticks, receive) = watch::channel(());
    try_join(
        wait_for_reports(delay, ticks, reconciliation),
        send_reports(agent, connection, receive, connected),
    )
    .await?;
    Ok(())
}

async fn wait_for_reports(
    delay: Arc<dyn Delay>,
    ticks: watch::Sender<()>,
    reconciliation: Arc<Notify>,
) -> Result<()> {
    loop {
        delay.wait(REPORT_INTERVAL_SECONDS).await;
        reconciliation.notify_one();
        ticks.send_replace(());
    }
}

async fn send_reports(
    agent: Arc<dyn Agent>,
    connection: Arc<dyn Connection>,
    mut ticks: watch::Receiver<()>,
    connected: watch::Sender<bool>,
) -> Result<()> {
    while ticks.changed().await.is_ok() {
        if !*connected.borrow() {
            continue;
        }
        let report = AgentMessage::Report {
            report: agent.report().await?,
        };
        if let Err(error) = connection.send(&report).await {
            tracing::warn!(error = %error, "report send failed; preserve VMs and reconnect");
            connected.send_replace(false);
        }
    }
    Ok(())
}
