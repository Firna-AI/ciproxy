//! Persist GitHub-only decisions before releasing a waiting Worker.

use ciproxy_protocol::guest::{AgentToGuest, JobDecision};
use ciproxy_protocol::runners::RunnerState;

use crate::cache_authority::CacheAuthority;
use crate::controller::Controller;
use crate::error::{Error, Result};
use crate::state::{PreparationState, VmRecord};

impl Controller {
    pub(crate) async fn release_pending_jobs(&self) -> Result<()> {
        let pending = {
            let mut state = self.state.lock().await;
            let mut next = state.clone();
            let mut pending = Vec::new();
            for vm in &mut next.runners {
                if matches!(vm.report.state, RunnerState::Running { .. })
                    && vm.job_facts.is_none()
                    && vm.preparation == PreparationState::Waiting
                {
                    vm.cache_authority = CacheAuthority::GitHubOnly;
                    pending.push(vm.clone());
                }
            }
            if *state != next {
                self.store.save(&next)?;
                *state = next;
            }
            pending
        };
        for vm in pending {
            self.release_github(&vm).await?;
        }
        Ok(())
    }

    pub(crate) async fn release_github(&self, vm: &VmRecord) -> Result<()> {
        let (updated, was_sealed) = {
            let mut state = self.state.lock().await;
            let mut next = state.clone();
            let record = next
                .runners
                .iter_mut()
                .find(|record| record.local_id == vm.local_id)
                .ok_or(Error::GuestMismatch)?;
            let was_sealed = matches!(record.cache_authority, CacheAuthority::Sealed(_));
            record.cache_authority = CacheAuthority::GitHubOnly;
            let updated = record.clone();
            if *state != next {
                self.store.save(&next)?;
                *state = next;
            }
            (updated, was_sealed)
        };
        if was_sealed
            && updated.job_facts.is_some()
            && let Err(error) = self.cache.fallback(&updated).await
        {
            tracing::warn!(%error, "GitHub-only cache cleanup will retry");
        }
        let RunnerState::Running { job_id } = &updated.report.state else {
            return Err(Error::GuestMismatch);
        };
        if let Err(error) = self
            .vms
            .send(
                &updated,
                &AgentToGuest::PrepareJob {
                    job_id: job_id.clone(),
                    decision: JobDecision::GitHubOnly,
                },
            )
            .await
        {
            tracing::warn!(%error, "host decision will replay after guest reconnect");
        }
        Ok(())
    }
}
