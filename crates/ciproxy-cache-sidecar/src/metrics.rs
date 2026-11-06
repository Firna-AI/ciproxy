//! Machine-observed restore reports, persisted across service restarts.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::cache::{
    CacheRestoreCounts, JobCacheReport, JobCacheUsage, MachineCacheReport, UnknownJobCount,
};
use ciproxy_protocol::identifiers::{JobId, OrganizationId, RepositoryId};
use ciproxy_protocol::jobs::JobAssignment;
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::runtime::CacheRegistration;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::clock::Clock;
use crate::error::{CacheError, Result};
use crate::storage::interface::Storage;

const WEEK: u64 = 7 * 24 * 60 * 60;

#[derive(Default, Deserialize, Serialize)]
struct Records {
    events: Vec<Event>,
    jobs: BTreeMap<String, Job>,
}

#[derive(Deserialize, Serialize)]
struct Event {
    time: u64,
    local: bool,
}

#[derive(Deserialize, Serialize)]
struct Job {
    job_id: JobId,
    organization_id: OrganizationId,
    repository_id: RepositoryId,
    assignment: JobAssignment,
    usage: JobCacheUsage,
    active: bool,
    updated: u64,
    #[serde(default)]
    unknown_reason: Option<UnknownJobScopeReason>,
}

#[async_trait]
pub(crate) trait Reports: Send + Sync {
    async fn registered(&self, registration: &CacheRegistration, scope: &JobScope);
    async fn closed(&self, registration: &CacheRegistration);
    async fn restored(&self, registration: &CacheRegistration, local: bool, bytes: u64);
    async fn fallback(&self, registration: &CacheRegistration);
    async fn report(&self) -> Result<MachineCacheReport>;
}

pub(crate) struct Metrics {
    store: Arc<dyn Storage>,
    clock: Arc<dyn Clock>,
    records: Mutex<Option<Records>>,
}

impl Metrics {
    pub fn new(store: Arc<dyn Storage>, clock: Arc<dyn Clock>) -> Self {
        Self {
            store,
            clock,
            records: Mutex::new(None),
        }
    }

    async fn load(&self) -> Records {
        let mut records: Records = match self.store.read_metrics().await {
            Ok(bytes) if !bytes.is_empty() => serde_json::from_slice(&bytes).unwrap_or_default(),
            _ => Records::default(),
        };
        for job in records.jobs.values_mut().filter(|job| job.active) {
            job.usage = JobCacheUsage::Github;
        }
        records
    }

    async fn persist(&self, records: &mut Records) {
        let cutoff = self.clock.now().saturating_sub(WEEK);
        records.events.retain(|event| event.time >= cutoff);
        records.jobs.retain(|_, job| job.updated >= cutoff);
        match serde_json::to_vec(records) {
            Ok(bytes) => {
                if self.store.write_metrics(&bytes).await.is_err() {
                    tracing::warn!("Cache report persistence unavailable");
                }
            }
            Err(_) => tracing::warn!("Cache report encoding unavailable"),
        }
    }
}

#[async_trait]
impl Reports for Metrics {
    async fn registered(&self, registration: &CacheRegistration, scope: &JobScope) {
        let mut lock = self.records.lock().await;
        if lock.is_none() {
            *lock = Some(self.load().await);
        }
        if let Some(records) = lock.as_mut() {
            let job = records.jobs.entry(key(registration)).or_insert(Job {
                job_id: registration.job_id.clone(),
                organization_id: registration.facts.organization_id.clone(),
                repository_id: registration.facts.repository.id.clone(),
                assignment: registration.facts.assignment.clone(),
                usage: JobCacheUsage::MachineHandled(CacheRestoreCounts {
                    local_entries: 0,
                    github_entries: 0,
                    restored_bytes: 0,
                }),
                active: true,
                updated: self.clock.now(),
                unknown_reason: match scope {
                    JobScope::Unknown { reason } => Some(*reason),
                    JobScope::Checked { .. } => None,
                },
            });
            if job.unknown_reason.is_none()
                && let JobScope::Unknown { reason } = scope
            {
                job.unknown_reason = Some(*reason);
            }
            self.persist(records).await;
        }
    }

    async fn closed(&self, registration: &CacheRegistration) {
        let mut lock = self.records.lock().await;
        if let Some(records) = lock.as_mut() {
            if let Some(job) = records.jobs.get_mut(&key(registration)) {
                job.active = false;
            }
            self.persist(records).await;
        }
    }

    async fn restored(&self, registration: &CacheRegistration, local: bool, bytes: u64) {
        let mut lock = self.records.lock().await;
        if lock.is_none() {
            *lock = Some(self.load().await);
        }
        if let Some(records) = lock.as_mut() {
            records.events.push(Event {
                time: self.clock.now(),
                local,
            });
            if let Some(job) = records.jobs.get_mut(&key(registration)) {
                if let JobCacheUsage::MachineHandled(counts) = &mut job.usage {
                    if local {
                        counts.local_entries += 1;
                    } else {
                        counts.github_entries += 1;
                    }
                    counts.restored_bytes += bytes;
                }
                job.updated = self.clock.now();
            }
            self.persist(records).await;
        }
    }

    async fn fallback(&self, registration: &CacheRegistration) {
        let mut lock = self.records.lock().await;
        if lock.is_none() {
            *lock = Some(self.load().await);
        }
        if let Some(records) = lock.as_mut() {
            let unknown_reason = records
                .jobs
                .get(&key(registration))
                .and_then(|job| job.unknown_reason);
            records.jobs.insert(
                key(registration),
                Job {
                    job_id: registration.job_id.clone(),
                    organization_id: registration.facts.organization_id.clone(),
                    repository_id: registration.facts.repository.id.clone(),
                    assignment: registration.facts.assignment.clone(),
                    usage: JobCacheUsage::Github,
                    active: true,
                    updated: self.clock.now(),
                    unknown_reason,
                },
            );
            self.persist(records).await;
        }
    }

    async fn report(&self) -> Result<MachineCacheReport> {
        let mut lock = self.records.lock().await;
        if lock.is_none() {
            *lock = Some(self.load().await);
        }
        let (used_bytes, capacity_bytes) = self.store.usage().await?;
        let records = lock.as_ref().ok_or(CacheError::Unavailable)?;
        let cutoff = self.clock.now().saturating_sub(WEEK);
        let events: Vec<_> = records
            .events
            .iter()
            .filter(|event| event.time >= cutoff)
            .collect();
        Ok(MachineCacheReport {
            used_bytes,
            capacity_bytes,
            restores_last_7_days: events.len() as u64,
            hits_last_7_days: events.iter().filter(|event| event.local).count() as u64,
            unknown_jobs: unknown_counts(records, cutoff),
            jobs: records
                .jobs
                .values()
                .map(|job| JobCacheReport {
                    job_id: job.job_id.clone(),
                    organization_id: job.organization_id.clone(),
                    repository_id: job.repository_id.clone(),
                    assignment: job.assignment.clone(),
                    usage: job.usage.clone(),
                })
                .collect(),
        })
    }
}

fn unknown_counts(records: &Records, cutoff: u64) -> Vec<UnknownJobCount> {
    let mut counts = BTreeMap::new();
    for job in records.jobs.values().filter(|job| job.updated >= cutoff) {
        if let Some(reason) = job.unknown_reason {
            *counts.entry(reason).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .map(|(reason, jobs)| UnknownJobCount { reason, jobs })
        .collect()
}

fn key(registration: &CacheRegistration) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        registration.facts.organization_id.0,
        registration.facts.repository.id.0,
        registration.facts.assignment.run_id,
        registration.facts.assignment.runner_id,
        registration.job_id.0
    )
}
