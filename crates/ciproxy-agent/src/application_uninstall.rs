//! Run the uninstall CLI while its required runtime ownership stays alive.

use std::sync::Arc;
use std::time::Duration;

use ciproxy_protocol::reports::REPORT_INTERVAL_SECONDS;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::boundaries::GuestEvent;
use crate::controller::Agent;
use crate::error::{Error, Result};
use crate::preparation::{Uninstall, UninstallState};

pub(crate) async fn execute(
    uninstall: Arc<dyn Uninstall>,
    agent: Arc<dyn Agent>,
    mut events: mpsc::Receiver<GuestEvent>,
) -> Result<()> {
    if uninstall.start().await? == UninstallState::Complete {
        println!("ciproxy agent uninstalled");
        return Ok(());
    }
    let mut timer = interval(Duration::from_secs(REPORT_INTERVAL_SECONDS));
    loop {
        match uninstall.finish().await {
            Ok(()) => break,
            Err(Error::Draining) => {}
            Err(error) => {
                tracing::warn!(%error, "Uninstall cleanup will retry automatically. Stop this command before running uninstall again.")
            }
        }
        let result = tokio::select! {
            event = events.recv() => {
                let event = event.ok_or(Error::Disconnected)?;
                if event.configuration_lost {
                    agent.configuration_lost(&event.runner_id.0).await?;
                }
                agent.guest(event.provenance, &event.runner_id.0, event.message).await.map(|_| ())
            }
            _ = timer.tick() => agent.reconcile().await,
        };
        if let Err(error) = result {
            tracing::warn!(error = %error, "uninstall is waiting for job and VM cleanup");
        }
    }
    println!("ciproxy agent uninstalled");
    Ok(())
}
