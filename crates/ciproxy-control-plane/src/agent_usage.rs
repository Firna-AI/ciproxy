//! Optional exit usage is saved separately from the authenticated lifecycle event.

use chrono::{DateTime, Utc};
use ciproxy_overflow::job_usage::UsageLimits;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::usage::JobUsage;
use ciproxy_store_interface::job_usage::JobUsageStore;
use uuid::Uuid;

use crate::runtime::MachineRuntime;
use crate::runtime_agent_event::parse_runner_id;

/// Usage removed from an exit message before lifecycle admission.
struct AgentExitUsage {
    runner_id: Uuid,
    usage: JobUsage,
}

impl AgentExitUsage {
    /// Extract usage only from a report with a canonical runner identity.
    fn take(message: &mut AgentMessage) -> Option<Self> {
        let AgentMessage::RunnerExited {
            runner_id, usage, ..
        } = message
        else {
            return None;
        };
        let runner_id = parse_runner_id(runner_id)?;
        let (usage, reason) = usage.take()?.into_parts();
        if let Some(reason) = reason {
            tracing::warn!(%reason, "dropped agent job usage");
        }
        Some(Self {
            runner_id,
            usage: usage?,
        })
    }

    /// Save valid usage for this machine. A failure does not affect the exit.
    fn save(
        self,
        store: &dyn JobUsageStore,
        workspace_id: Uuid,
        machine_id: Uuid,
        received_at: DateTime<Utc>,
    ) {
        let target = match store.own_usage_target(workspace_id, machine_id, self.runner_id) {
            Ok(Some(target)) => target,
            Ok(None) => {
                tracing::warn!("dropped agent job usage: runner is not on this machine");
                return;
            }
            Err(_) => {
                tracing::warn!("dropped agent job usage: runner lookup failed");
                return;
            }
        };
        let limits = UsageLimits {
            vcpus: target.vcpus,
            memory_bytes: target.memory_bytes,
            started_at: target.started_at,
            received_at,
        };
        if limits
            .save_final(store, workspace_id, self.runner_id, self.usage)
            .is_err()
        {
            tracing::warn!("dropped final agent job usage: store write failed");
        }
    }
}

impl MachineRuntime {
    /// Prepare the same usage write for queued and direct agent exits.
    pub(crate) fn prepare_agent_usage(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        message: &mut AgentMessage,
    ) -> Option<impl FnOnce() + Send + use<>> {
        let usage = AgentExitUsage::take(message)?;
        let store = self.store.clone();
        let received_at = self.clock.now();
        Some(move || usage.save(store.as_ref(), workspace_id, machine_id, received_at))
    }
}
