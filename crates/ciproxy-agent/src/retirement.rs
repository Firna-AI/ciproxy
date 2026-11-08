//! Claim idle stops before proving configuration loss or touching VM resources.

use ciproxy_protocol::runners::RunnerState;

use crate::controller::Controller;
use crate::error::Result;
use crate::registration::RegistrationAction;
use crate::state::{RegistrationClaim, VmRecord};

impl Controller {
    pub(crate) async fn request_idle_stop(&self, vm: &VmRecord) -> Result<()> {
        match self
            .claim_registration(vm, RegistrationClaim::Retirement)
            .await?
        {
            RegistrationAction::Preserve => Ok(()),
            RegistrationAction::GuestShutdown(vm) => {
                self.stop_claimed(&vm, RegistrationClaim::Delivery).await
            }
            RegistrationAction::Claimed(vm) => {
                let result = self.stop_claimed(&vm, RegistrationClaim::Retirement).await;
                self.release_registration(&vm, RegistrationClaim::Retirement)
                    .await;
                result
            }
        }
    }

    async fn stop_claimed(&self, vm: &VmRecord, owner: RegistrationClaim) -> Result<()> {
        let lost = if vm.report.configuration_lost {
            true
        } else {
            match self.vms.configuration_lost(vm).await {
                Ok(lost) => lost,
                Err(error) => {
                    tracing::warn!(%error, "configuration proof unavailable; keep guest shutdown");
                    false
                }
            }
        };
        if owner == RegistrationClaim::Delivery || !lost {
            return self.vms.request_stop(vm).await;
        }
        self.record_configuration(&vm.report.runner_id.0, true)
            .await?;
        let current = self
            .state
            .lock()
            .await
            .runners
            .iter()
            .find(|record| {
                record.local_id == vm.local_id
                    && !matches!(record.report.state, RunnerState::Running { .. })
                    && record.report.configuration_lost
            })
            .cloned();
        if let Some(vm) = current {
            self.retire(&vm).await?;
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

#[cfg(test)]
#[path = "_tests_/registration_recovery_tests.rs"]
mod registration_recovery_tests;
