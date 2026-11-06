//! Scale-set lock ownership and durable notification persistence.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_store_interface::error;
use ciproxy_store_interface::github::{GithubJob, JobDelivery, RunFacts};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetSession, ScaleSetState};
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};

use super::{StoreAdapter, query};

/// A workspace-scoped scale-set identity supplied by runtime orchestration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListenerTarget {
    /// The owning workspace.
    pub workspace_id: Uuid,
    /// The organization registration.
    pub organization_id: Uuid,
    /// The internal scale-set identity.
    pub scale_set_id: Uuid,
}

/// Context read from a tenant-scoped store, independent of caller-supplied names.
#[derive(Clone, Debug)]
pub struct ListenerContext {
    /// The current organization installation and state.
    pub organization: Organization,
    /// The registered organization-label pair.
    pub scale_set: ScaleSet,
    /// Routing and confirmed remote cleanup state of this exact binding.
    pub state: ScaleSetState,
    /// Unfinished work or an unconfirmed delivery still needs this listener.
    pub has_tracked_jobs: bool,
    /// The organization has completed local admission for new execution.
    pub organization_enabled: bool,
}

/// A live database session lock held for a whole GitHub message session.
#[cfg_attr(test, unimock::unimock(api = SessionLeaseMock))]
#[async_trait]
pub trait SessionLease: Send + Sync {
    /// Fail when the underlying connection no longer owns its lock.
    async fn check(&self) -> Result<()>;
}

/// The persistence seam used by the scale-set listener.
#[cfg_attr(test, unimock::unimock(api = SessionStoreMock))]
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Read saved trusted facts before requesting metadata for a known run again.
    async fn facts(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        run_id: i64,
    ) -> Result<Option<RunFacts>>;
    /// Read current organization and scale-set context under workspace scope.
    async fn context(&self, target: ListenerTarget) -> Result<ListenerContext>;
    /// Try to own the scale set's dedicated database session.
    async fn lock(&self, target: ListenerTarget) -> Result<Option<Box<dyn SessionLease>>>;
    /// Persist one monotonic job update and its delivery revision.
    async fn save(&self, workspace_id: Uuid, job: GithubJob) -> Result<()>;
    /// Read notifications that remain pending across listener restarts.
    async fn pending(&self, target: ListenerTarget) -> Result<Vec<JobDelivery>>;
    /// Confirm the revision that the output accepted.
    async fn confirm(&self, target: ListenerTarget, job_id: Uuid, revision: i64) -> Result<()>;
}

struct Lease {
    handle: Arc<Mutex<Box<dyn ScaleSetSession>>>,
}

#[async_trait]
impl SessionLease for Lease {
    async fn check(&self) -> Result<()> {
        let handle = self.handle.clone();
        tokio::task::spawn_blocking(move || {
            let Ok(mut handle) = handle.lock() else {
                return Err(Error::OrganizationInactive);
            };
            handle.check_connection().internal()
        })
        .await
        .internal()?
    }
}

#[async_trait]
impl SessionStore for StoreAdapter {
    async fn facts(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        run_id: i64,
    ) -> Result<Option<RunFacts>> {
        query(self.store.clone(), move |store| {
            store.github_run_facts(workspace_id, organization_id, run_id)
        })
        .await
    }
    async fn context(&self, target: ListenerTarget) -> Result<ListenerContext> {
        query(self.store.clone(), move |store| {
            let organization = store
                .organization(target.workspace_id, target.organization_id)?
                .ok_or(error::Error::NotFound)?;
            let binding = store
                .scale_set_binding(
                    target.workspace_id,
                    target.organization_id,
                    target.scale_set_id,
                )?
                .ok_or(error::Error::NotFound)?;
            Ok(ListenerContext {
                organization_enabled: store
                    .organization_runnable(target.workspace_id, target.organization_id)?,
                organization,
                scale_set: binding.scale_set,
                state: binding.state,
                has_tracked_jobs: binding.has_tracked_jobs,
            })
        })
        .await
    }
    async fn lock(&self, target: ListenerTarget) -> Result<Option<Box<dyn SessionLease>>> {
        let handle = query(self.store.clone(), move |store| {
            store.try_lock_scale_set(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id,
            )
        })
        .await?;
        Ok(handle.map(|handle| {
            Box::new(Lease {
                handle: Arc::new(Mutex::new(handle)),
            }) as Box<dyn SessionLease>
        }))
    }
    async fn save(&self, workspace_id: Uuid, job: GithubJob) -> Result<()> {
        let observed_at = self.runtime.now();
        query(self.store.clone(), move |store| {
            store.save_github_job(workspace_id, &job, observed_at)
        })
        .await
    }
    async fn pending(&self, target: ListenerTarget) -> Result<Vec<JobDelivery>> {
        query(self.store.clone(), move |store| {
            store.pending_github_jobs(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id,
            )
        })
        .await
    }
    async fn confirm(&self, target: ListenerTarget, job_id: Uuid, revision: i64) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.confirm_github_job(
                target.workspace_id,
                target.organization_id,
                job_id,
                revision,
            )
        })
        .await
    }
}
