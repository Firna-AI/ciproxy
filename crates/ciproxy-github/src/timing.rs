//! Recover real job timestamps by unique ephemeral runner, with bounded retry.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::github_timing::{JobTimes, PendingTiming};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::Result;
use crate::persistence::{StoreAdapter, query};
use crate::rest::GitHubRest;
use crate::runtime::Runtime;

#[cfg_attr(test,unimock::unimock(api=TimingStoreMock))]
#[async_trait]
pub trait TimingStore: Send + Sync {
    /// Discover missing real times and the exact installation/repository authority.
    async fn pending(&self, workspace_id: Uuid) -> Result<Vec<TimingTarget>>;
    /// Save authoritative API facts without replacing the exact attempt identity.
    async fn save(
        &self,
        workspace_id: Uuid,
        attempt_id: Uuid,
        times: JobTimes,
        at: DateTime<Utc>,
    ) -> Result<()>;
}

/// Inputs come only from durable assignments and installation API snapshots.
pub struct TimingTarget {
    /// Exact pending attempt and raw demand proof.
    pub attempt: PendingTiming,
    /// Current installation with existing Actions-read authority.
    pub installation_id: i64,
    /// Current visible repository name for this stable repository ID.
    pub repository: String,
}

#[async_trait]
impl TimingStore for StoreAdapter {
    async fn pending(&self, workspace_id: Uuid) -> Result<Vec<TimingTarget>> {
        query(self.store.clone(), move |store| {
            let mut output = Vec::new();
            for attempt in store.incomplete_github_job_attempts(workspace_id)? {
                let Some(organization) =
                    store.organization(workspace_id, attempt.job.organization_id)?
                else {
                    continue;
                };
                if organization.app_state
                    == ciproxy_store_interface::organizations::AppState::Removed
                {
                    continue;
                }
                let repository = store
                    .repositories(workspace_id, organization.id)?
                    .into_iter()
                    .find(|repository| repository.id == attempt.job.facts.repository.id)
                    .map_or_else(
                        || attempt.job.facts.repository.full_name.clone(),
                        |repository| repository.full_name,
                    );
                output.push(TimingTarget {
                    attempt,
                    installation_id: organization.installation_id,
                    repository,
                });
            }
            Ok(output)
        })
        .await
    }
    async fn save(
        &self,
        workspace_id: Uuid,
        attempt_id: Uuid,
        times: JobTimes,
        at: DateTime<Utc>,
    ) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.save_github_job_times(workspace_id, attempt_id, &times, at)
        })
        .await
    }
}

/// Timing recovery never invents starts, completions, or numeric job IDs.
#[unimock::unimock(api = JobTimingMock)]
#[async_trait]
pub trait JobTiming: Send + Sync {
    /// Retry pending attempts after their bounded backoff interval.
    async fn repair(&self, workspace_id: Uuid) -> Result<()>;
}

struct Retry {
    not_before: DateTime<Utc>,
    seconds: i64,
}

/// Metadata-only recovery with original App permission and injected time.
pub struct RepairTiming {
    store: Arc<dyn TimingStore>,
    rest: Arc<dyn GitHubRest>,
    clock: Arc<dyn Runtime>,
    retries: Mutex<HashMap<Uuid, Retry>>,
}

impl RepairTiming {
    /// Construct at the composition root with existing API/store adapters.
    pub fn new(
        store: Arc<dyn TimingStore>,
        rest: Arc<dyn GitHubRest>,
        clock: Arc<dyn Runtime>,
    ) -> Self {
        Self {
            store,
            rest,
            clock,
            retries: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl JobTiming for RepairTiming {
    async fn repair(&self, workspace_id: Uuid) -> Result<()> {
        let now = self.clock.now();
        for target in self.store.pending(workspace_id).await? {
            let id = target.attempt.attempt_id;
            if self
                .retries
                .lock()
                .await
                .get(&id)
                .is_some_and(|retry| now < retry.not_before)
            {
                continue;
            }
            let job = target.attempt.job;
            let Some(runner_id) = job.runner_github_id else {
                continue;
            };
            let complete = match self
                .rest
                .job_times(
                    target.installation_id,
                    &target.repository,
                    job.run_id,
                    runner_id,
                )
                .await
            {
                Ok(Some(times)) => {
                    let complete = times.started_at.or(job.started_at).is_some()
                        && (!(times.terminal || job.phase == DemandPhase::Completed)
                            || times.completed_at.or(job.completed_at).is_some());
                    self.store.save(workspace_id, id, times, now).await?;
                    complete
                }
                result => {
                    if let Err(error) = result {
                        tracing::warn!(%id,%error,"real job timing lookup will retry");
                    }
                    false
                }
            };
            let mut retries = self.retries.lock().await;
            if complete {
                retries.remove(&id);
            } else {
                let seconds = retries
                    .get(&id)
                    .map_or(30, |retry| (retry.seconds * 2).min(300));
                retries.insert(
                    id,
                    Retry {
                        not_before: now + chrono::Duration::seconds(seconds),
                        seconds,
                    },
                );
            }
        }
        Ok(())
    }
}
