//! Claim registration delivery and idle retirement under the machine state lock.

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::runners::{RunnerSpec, RunnerState};
use ciproxy_protocol::version::ProtocolVersion;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::{Mode, RegistrationClaim, VmRecord};

pub(crate) enum RegistrationAction {
    Claimed(VmRecord),
    GuestShutdown(VmRecord),
    Preserve,
}

impl Controller {
    /// Reserve the VM before connection proof or VM work can wait.
    /// A durable stop also refuses delivery after the in-flight claim ends.
    pub(crate) async fn claim_registration(
        &self,
        vm: &VmRecord,
        claim: RegistrationClaim,
    ) -> Result<RegistrationAction> {
        let mut state = self.state.lock().await;
        let Some(index) = state
            .runners
            .iter()
            .position(|record| record.local_id == vm.local_id)
        else {
            return match claim {
                RegistrationClaim::Delivery => Err(Error::RunnerMismatch),
                RegistrationClaim::Retirement => Ok(RegistrationAction::Preserve),
            };
        };
        let active = state.registration_claims.get(&vm.local_id).copied();
        let before = &state.runners[index].report;
        let mut next = state.clone();
        let record = &mut next.runners[index];
        let action = match claim {
            RegistrationClaim::Delivery => {
                match record.report.state {
                    RunnerState::Stopping => return Err(Error::RunnerMismatch),
                    RunnerState::Warm | RunnerState::Running { .. } => {
                        return Ok(RegistrationAction::Preserve);
                    }
                    RunnerState::Starting => {}
                }
                if state.mode != Mode::Active {
                    return Err(Error::Draining);
                }
                if active.is_some() {
                    return Ok(RegistrationAction::Preserve);
                }
                record.report.configuration_lost = false;
                RegistrationAction::Claimed(record.clone())
            }
            RegistrationClaim::Retirement => {
                if matches!(record.report.state, RunnerState::Running { .. })
                    || active == Some(RegistrationClaim::Retirement)
                {
                    return Ok(RegistrationAction::Preserve);
                }
                record.report.state = RunnerState::Stopping;
                if active == Some(RegistrationClaim::Delivery) {
                    RegistrationAction::GuestShutdown(record.clone())
                } else {
                    RegistrationAction::Claimed(record.clone())
                }
            }
        };
        if matches!(action, RegistrationAction::Claimed(_)) {
            next.registration_claims.insert(vm.local_id.clone(), claim);
        }
        if next.runners[index].report != *before {
            self.store.save(&next)?;
        }
        *state = next;
        Ok(action)
    }

    pub(crate) async fn release_registration(&self, vm: &VmRecord, claim: RegistrationClaim) {
        let mut state = self.state.lock().await;
        if state.registration_claims.get(&vm.local_id) == Some(&claim) {
            state.registration_claims.remove(&vm.local_id);
        }
    }

    pub(crate) async fn retry_start(&self, vm: VmRecord, runner: RunnerSpec) -> Result<()> {
        if vm.report.organization_id != runner.organization_id
            || vm.report.label != runner.label
            || vm.report.requested_size != runner.size
            || runner.slot_indices != vm.report.slot_indices
        {
            return Err(Error::RunnerMismatch);
        }
        let vm = match self
            .claim_registration(&vm, RegistrationClaim::Delivery)
            .await?
        {
            RegistrationAction::Claimed(vm) => vm,
            RegistrationAction::Preserve => return Ok(()),
            RegistrationAction::GuestShutdown(_) => return Err(Error::RunnerMismatch),
        };
        let result = self.vms.send(&vm, &configure(runner)).await;
        self.release_registration(&vm, RegistrationClaim::Delivery)
            .await;
        result
    }
}

pub(crate) fn configure(runner: RunnerSpec) -> AgentToGuest {
    AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: runner.runner_id,
        organization_id: runner.organization_id,
        registration: runner.registration,
        cache: None,
    }
}
