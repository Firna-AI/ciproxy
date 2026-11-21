//! One lifecycle owner keeps active and pending network indices unavailable.

use ciproxy_protocol::runners::RunnerState;

use crate::controller::Controller;
use crate::error::Result;
use crate::state::VmRecord;

impl Controller {
    /// Transfer durable ownership before releasing the completed VM report.
    pub(crate) async fn finish_vm_cleanup(&self, vm: &VmRecord) -> Result<()> {
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        next.runners.retain(|record| record.local_id != vm.local_id);
        next.failed_starts.remove(&vm.local_id);
        next.pending_networks.insert(vm.network_index);
        self.store.save(&next)?;
        *state = next;
        Ok(())
    }

    /// Keep a failed start terminal until reconciliation removes its VM files.
    pub(crate) async fn mark_failed_cleanup(&self, vm: &VmRecord) -> Result<()> {
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        if let Some(record) = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
        {
            if !matches!(record.report.state, RunnerState::Running { .. }) {
                record.report.state = RunnerState::Stopping;
            }
            next.failed_starts.insert(vm.local_id.clone());
            self.store.save(&next)?;
            *state = next;
        }
        Ok(())
    }

    /// Adopt reserved-name orphans and release indices only after checked cleanup.
    pub(crate) async fn retry_network_cleanup(&self) -> Result<()> {
        let _cleanup = self.network_cleanup.lock().await;
        let owned = self.vms.networks().await?;
        let pending = {
            let mut state = self.state.lock().await;
            let mut next = state.clone();
            for index in owned {
                if !state.runners.iter().any(|vm| vm.network_index == index) {
                    next.pending_networks.insert(index);
                }
            }
            if next != *state {
                self.store.save(&next)?;
                *state = next;
            }
            state.pending_networks.clone()
        };
        let mut failure = None;
        for index in pending {
            if let Err(error) = self.vms.remove_network(index).await {
                tracing::warn!(%error, index, "owned network cleanup remains pending");
                if failure.is_none() {
                    failure = Some(error);
                }
                continue;
            }
            let mut state = self.state.lock().await;
            let mut next = state.clone();
            next.pending_networks.remove(&index);
            self.store.save(&next)?;
            *state = next;
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
