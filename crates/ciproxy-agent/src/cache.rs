//! Authenticated host-only cache administration and in-memory job handoffs.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::cache::{CacheConnection, MachineCacheReport};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use ciproxy_protocol::runtime::{CacheRegistration, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use tokio::sync::Mutex;

use crate::boundaries::Cache;
use crate::cache_admin::{
    Administration, CacheFiles, Credential, HttpAdministration, OwnedCacheFiles,
};
use crate::error::{Error, Result};
use crate::network::host_address;
use crate::process::{Commands, UnitState, arguments};
use crate::state::VmRecord;

struct Pending {
    vm: VmRecord,
    job_id: JobId,
    runtime: RuntimeCredentials,
    session: Option<Credential>,
    fallback: bool,
}

pub(crate) struct HttpCache {
    admin: Arc<dyn Administration>,
    files: Arc<dyn CacheFiles>,
    pub(crate) commands: Arc<dyn Commands>,
    pending: Mutex<HashMap<String, Pending>>,
}

impl HttpCache {
    pub(crate) fn new(commands: Arc<dyn Commands>) -> Result<Self> {
        Ok(Self::with_boundaries(
            Arc::new(HttpAdministration::new(
                "/var/lib/ciproxy/cache-admin-token".into(),
            )?),
            Arc::new(OwnedCacheFiles {
                directory: "/var/lib/ciproxy/area/cache".into(),
                commands: commands.clone(),
            }),
            commands,
        ))
    }

    pub(crate) fn with_boundaries(
        admin: Arc<dyn Administration>,
        files: Arc<dyn CacheFiles>,
        commands: Arc<dyn Commands>,
    ) -> Self {
        Self {
            admin,
            files,
            commands,
            pending: Mutex::new(HashMap::new()),
        }
    }

    async fn stop_service(&self) -> Result<()> {
        if self.commands.unit("ciproxy-cache.service").await? == UnitState::Active {
            self.commands
                .run(
                    "systemctl",
                    &arguments(&["stop", "ciproxy-cache.service"]),
                    &[],
                )
                .await?;
        }
        let mut pending = self.pending.lock().await;
        for job in pending.values_mut() {
            job.session = None;
        }
        Ok(())
    }
}

#[async_trait]
impl Cache for HttpCache {
    async fn refresh_budget(&self, capacity_bytes: u64) -> Result<()> {
        if let Err(error) = self.admin.budget(capacity_bytes).await {
            if let Err(restart) = self
                .commands
                .run(
                    "systemctl",
                    &arguments(&["start", "ciproxy-cache.service"]),
                    &[],
                )
                .await
            {
                tracing::warn!(%restart, "cache service recovery will retry without clearing entries");
            }
            return Err(error);
        }
        Ok(())
    }
    async fn budget(&self, capacity_bytes: u64) -> Result<()> {
        if self.admin.budget(capacity_bytes).await.is_ok() {
            return Ok(());
        }
        self.stop_service().await?;
        self.files.release_occupied_space().await?;
        if let Err(error) = self
            .commands
            .run(
                "systemctl",
                &arguments(&["start", "ciproxy-cache.service"]),
                &[],
            )
            .await
        {
            tracing::warn!(%error, "cache restart will retry; reclaimed space remains available to jobs");
        }
        tracing::warn!("machine cache stopped and occupied cache space released; jobs use GitHub");
        Ok(())
    }

    async fn report(&self) -> Result<MachineCacheReport> {
        self.admin.report().await
    }
    async fn clear(&self) -> Result<()> {
        self.admin.clear().await
    }

    async fn handoff(
        &self,
        vm: &VmRecord,
        job_id: &JobId,
        runtime: RuntimeCredentials,
    ) -> Result<()> {
        let mut pending = self.pending.lock().await;
        pending
            .entry(vm.report.runner_id.0.clone())
            .or_insert_with(|| Pending {
                vm: vm.clone(),
                job_id: job_id.clone(),
                runtime,
                session: None,
                fallback: false,
            });
        Ok(())
    }

    async fn prepare(
        &self,
        runner: &RunnerId,
        facts: &JobFacts,
    ) -> Result<Option<(CacheConnection, JobScope, u64)>> {
        let mut pending = self.pending.lock().await;
        let Some(job) = pending.get_mut(&runner.0) else {
            return Ok(None);
        };
        if job.fallback {
            return Ok(None);
        }
        if facts.assignment.job_id.as_ref() != Some(&job.job_id) {
            return Err(Error::GuestMismatch);
        }
        if job.runtime.proxy_url.is_empty() {
            return Ok(None);
        }
        {
            let registration = CacheRegistration {
                job_id: job.job_id.clone(),
                facts: facts.clone(),
                permissions: job.runtime.permissions,
                github_token: job.runtime.github_token.clone(),
                cache_url: job.runtime.cache_url.clone(),
                results_url: job.runtime.results_url.clone(),
                proxy_url: job.runtime.proxy_url.clone(),
            };
            job.session = Some(match self.admin.recover(&registration).await? {
                Some(session) => session,
                None => self.admin.register(registration).await?,
            });
        }
        let session = job.session.as_ref().ok_or(Error::CacheUnavailable)?;
        Ok(Some((
            CacheConnection {
                url: Secret::new(format!(
                    "http://{}:7887",
                    host_address(job.vm.network_index)
                )),
                token: session.token.clone(),
            },
            session.scope.clone(),
            job.runtime.docker_data_bytes,
        )))
    }

    async fn fallback(&self, vm: &VmRecord) -> Result<()> {
        let Some(facts) = assigned_job(vm) else {
            return Ok(());
        };
        if let Some(job) = self.pending.lock().await.get_mut(&vm.report.runner_id.0) {
            job.fallback = true;
        }
        if self.admin.fallback(&facts).await.is_err() {
            self.stop_service().await?;
            if let Err(error) = self
                .commands
                .run(
                    "systemctl",
                    &arguments(&["start", "ciproxy-cache.service"]),
                    &[],
                )
                .await
            {
                tracing::warn!(%error,"cache sessions invalidated; restart will retry");
            }
            return Ok(());
        }
        self.admin.revoke(&facts).await
    }

    async fn release(&self, vm: &VmRecord) -> Result<()> {
        self.pending.lock().await.remove(&vm.report.runner_id.0);
        if let Some(facts) = assigned_job(vm)
            && self.admin.revoke(&facts).await.is_err()
        {
            self.stop_service().await?;
        }
        Ok(())
    }
}

fn assigned_job(vm: &VmRecord) -> Option<JobFacts> {
    let facts = vm.job_facts.as_ref()?;
    let job_id = facts.assignment.job_id.as_ref()?;
    (facts.organization_id == vm.report.organization_id
        && vm.report.state
            == (ciproxy_protocol::runners::RunnerState::Running {
                job_id: job_id.clone(),
            }))
    .then(|| facts.clone())
}

#[cfg(test)]
#[path = "_tests_/cache_tests.rs"]
mod cache_tests;
