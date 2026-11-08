//! End proven unconfigured VMs on the host and preserve configured guest stops.

use ciproxy_protocol::runners::RunnerState;

use crate::controller::Controller;
use crate::error::Result;
use crate::state::VmRecord;

impl Controller {
    pub(crate) async fn request_idle_stop(&self, vm: &VmRecord) -> Result<()> {
        let vm = {
            let mut state = self.state.lock().await;
            let mut next = state.clone();
            let Some(record) = next
                .runners
                .iter_mut()
                .find(|record| record.local_id == vm.local_id)
            else {
                return Ok(());
            };
            if matches!(record.report.state, RunnerState::Running { .. }) {
                return Ok(());
            }
            if record.report.configuration_lost {
                record.report.state = RunnerState::Stopping;
            }
            let vm = record.clone();
            if next != *state {
                self.store.save(&next)?;
                *state = next;
            }
            vm
        };
        if vm.report.configuration_lost {
            return self.retire(&vm).await;
        }
        self.vms.request_stop(&vm).await?;
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        if let Some(record) = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
            && !matches!(
                record.report.state,
                RunnerState::Running { .. } | RunnerState::Stopping
            )
        {
            record.report.state = RunnerState::Stopping;
            self.store.save(&next)?;
            *state = next;
        }
        Ok(())
    }

    pub(crate) async fn retire(&self, vm: &VmRecord) -> Result<()> {
        self.vms.stop(vm).await?;
        if let Some(dockers) = &self.dockers {
            if dockers.pending(vm)? {
                dockers.discard(vm).await?;
            }
            dockers.release(vm)?;
        }
        self.cache.release(vm).await?;
        self.finish_vm_cleanup(vm).await
    }
}
