//! Shared asynchronous ownership of dedicated store lease connections.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_store_interface::error;
use ciproxy_store_interface::github::OrganizationLease;
use ciproxy_store_interface::own_runners::OwnRunnerLease;
use ciproxy_store_interface::scale_sets::ScaleSetSession;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};

use super::query;
use super::session::ListenerTarget;

/// Dedicated store authority retained across asynchronous service operations.
#[cfg_attr(test, unimock::unimock(api = LeaseMock))]
#[async_trait]
pub trait Lease: Send + Sync {
    /// Stop continued use when the dedicated connection loses its authority.
    async fn check(&self) -> Result<()>;
}

/// Acquire service leases without blocking an asynchronous runtime worker.
#[async_trait]
pub trait LeaseStore: Send + Sync {
    /// Try to serialize execution decisions for one workspace.
    async fn own_runner_lease(&self, workspace_id: Uuid) -> Result<Option<Box<dyn Lease>>>;
    /// Wait for mutation authority over one full GitHub organization identity.
    async fn organization_lease(&self, github_id: i64) -> Result<Box<dyn Lease>>;
    /// Try to own one scale set for a complete GitHub message session.
    async fn scale_set_lease(&self, target: ListenerTarget) -> Result<Option<Box<dyn Lease>>>;
}

#[cfg_attr(test, unimock::unimock(api = ConnectionMock))]
trait Connection: Send {
    fn check_connection(&mut self) -> error::Result<()>;
}

enum StoreLease {
    OwnRunner(Box<dyn OwnRunnerLease>),
    Organization(Box<dyn OrganizationLease>),
    ScaleSet(Box<dyn ScaleSetSession>),
}

impl Connection for StoreLease {
    fn check_connection(&mut self) -> error::Result<()> {
        match self {
            Self::OwnRunner(lease) => lease.check_connection(),
            Self::Organization(lease) => lease.check_connection(),
            Self::ScaleSet(lease) => lease.check_connection(),
        }
    }
}

struct Handle {
    connection: Arc<Mutex<Box<dyn Connection>>>,
}

impl Handle {
    fn new(connection: Box<dyn Connection>) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
        }
    }
}

#[async_trait]
impl Lease for Handle {
    async fn check(&self) -> Result<()> {
        let connection = self.connection.clone();
        tokio::task::spawn_blocking(move || {
            let Ok(mut connection) = connection.lock() else {
                return Err(Error::OrganizationInactive);
            };
            connection.check_connection().internal()
        })
        .await
        .internal()?
    }
}

#[async_trait]
impl LeaseStore for Arc<dyn Store> {
    async fn own_runner_lease(&self, workspace_id: Uuid) -> Result<Option<Box<dyn Lease>>> {
        let lease = query(self.clone(), move |store| {
            store.try_lock_own_runners(workspace_id)
        })
        .await?;
        Ok(lease.map(|lease| {
            Box::new(Handle::new(Box::new(StoreLease::OwnRunner(lease)))) as Box<dyn Lease>
        }))
    }

    async fn organization_lease(&self, github_id: i64) -> Result<Box<dyn Lease>> {
        let lease = query(self.clone(), move |store| {
            store.lock_github_organization(github_id)
        })
        .await?;
        Ok(Box::new(Handle::new(Box::new(StoreLease::Organization(
            lease,
        )))))
    }

    async fn scale_set_lease(&self, target: ListenerTarget) -> Result<Option<Box<dyn Lease>>> {
        let lease = query(self.clone(), move |store| {
            store.try_lock_scale_set(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id,
            )
        })
        .await?;
        Ok(lease.map(|lease| {
            Box::new(Handle::new(Box::new(StoreLease::ScaleSet(lease)))) as Box<dyn Lease>
        }))
    }
}

#[cfg(test)]
#[path = "_tests_/lease_tests.rs"]
mod lease_tests;
