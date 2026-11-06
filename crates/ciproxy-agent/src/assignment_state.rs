//! Persist only trusted assignment metadata and original non-secret permissions.

use ciproxy_protocol::guest::JobPreparation;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::runtime::CachePermissions;

use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::{PreparationState, VmRecord};

impl Controller {
    pub(crate) async fn begin_preparation(&self, vm: &VmRecord) -> Result<bool> {
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        let record = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
            .ok_or(Error::GuestMismatch)?;
        if record.preparation != PreparationState::Waiting {
            return Ok(false);
        }
        record.preparation = PreparationState::Started;
        self.store.save(&next)?;
        *state = next;
        Ok(true)
    }

    pub(crate) async fn prepared_record(
        &self,
        vm: &VmRecord,
        outcome: JobPreparation,
    ) -> Result<()> {
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        let record = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
            .ok_or(Error::GuestMismatch)?;
        if record.preparation == PreparationState::Completed(outcome) {
            return Ok(());
        }
        record.preparation = PreparationState::Completed(outcome);
        self.store.save(&next)?;
        *state = next;
        Ok(())
    }

    pub(crate) async fn started_record(
        &self,
        vm: &VmRecord,
        job_id: &JobId,
        permissions: Option<CachePermissions>,
    ) -> Result<VmRecord> {
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        let record = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
            .ok_or(Error::GuestMismatch)?;
        if let RunnerState::Running { job_id: old } = &record.report.state
            && old != job_id
        {
            return Err(Error::GuestMismatch);
        }
        record.report.state = RunnerState::Running {
            job_id: job_id.clone(),
        };
        if record.cache_permissions.is_none() {
            record.cache_permissions = permissions;
        }
        let updated = record.clone();
        self.store.save(&next)?;
        *state = next;
        tracing::info!(event="job_started_observed",vm_id=%vm.local_id,"accepted guest job state");
        Ok(updated)
    }

    pub(crate) async fn trusted_facts(&self, vm: &VmRecord, facts: JobFacts) -> Result<JobFacts> {
        if let Some(original) = &vm.job_facts {
            if original.assignment != facts.assignment
                || original.organization_id != facts.organization_id
                || original.repository.id != facts.repository.id
            {
                return Err(Error::GuestMismatch);
            }
            return Ok(original.clone());
        }
        let mut state = self.state.lock().await;
        let mut next = state.clone();
        let record = next
            .runners
            .iter_mut()
            .find(|record| record.local_id == vm.local_id)
            .ok_or(Error::GuestMismatch)?;
        record.job_facts = Some(facts.clone());
        self.store.save(&next)?;
        *state = next;
        Ok(facts)
    }
}
