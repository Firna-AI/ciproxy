//! Bind each metadata request to the token workspace and exact assigned machine.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_github::rest::cache_metadata::CacheEntries;
use ciproxy_github::runtime::Runtime;
use ciproxy_protocol::cache_metadata::{CacheMetadataRequest, CacheMetadataResponse};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::TokenSubject;
use sha2::{Digest, Sha256};

use crate::assignment::Assignment;
use crate::error::{Error, ErrorResultExt, Result};

use super::records::MetadataRecords;

/// Authenticate a machine and return current metadata for its exact assignment.
#[async_trait]
pub(crate) trait CacheMetadata: Send + Sync {
    /// Resolve repository authority from the credential and durable job placement.
    async fn lookup(
        &self,
        credential: Secret,
        request: CacheMetadataRequest,
    ) -> Result<CacheMetadataResponse>;
}

/// Hosted orchestration consumed through the authenticated metadata boundary.
pub(crate) struct HostedCacheMetadata {
    /// Durable token and workspace placement facts.
    pub(crate) records: Arc<dyn MetadataRecords>,
    /// GitHub App cache listing, independent of all job requests.
    pub(crate) caches: Arc<dyn CacheEntries>,
    /// Trusted time used for token expiration.
    pub(crate) clock: Arc<dyn Runtime>,
}

#[async_trait]
impl CacheMetadata for HostedCacheMetadata {
    async fn lookup(
        &self,
        credential: Secret,
        request: CacheMetadataRequest,
    ) -> Result<CacheMetadataResponse> {
        let hash = TokenHash::new(Sha256::digest(credential.expose_secret().as_bytes()).to_vec());
        let token = self
            .records
            .authenticate(hash, self.clock.now())
            .await?
            .ok_or(Error::RunnerOwnership)?;
        let TokenSubject::Machine(machine_id) = token.subject else {
            return Err(Error::RunnerOwnership);
        };
        let snapshot = self.records.snapshot(token.workspace_id).await?;
        if snapshot.workspace_id != token.workspace_id {
            return Err(Error::RunnerOwnership);
        }
        let assignment = Assignment::for_worker(&snapshot, machine_id, &request.job_id)
            .ok_or(Error::RunnerOwnership)?;
        let job = assignment.job;
        let runner = assignment.runner;
        let organization = snapshot
            .organizations
            .iter()
            .find(|organization| {
                organization.id == runner.runner.organization_id
                    && organization.workspace_id == token.workspace_id
                    && organization.app_state == AppState::Active
            })
            .ok_or(Error::RunnerOwnership)?;
        let entries = self
            .caches
            .cache_entries(
                organization.installation_id,
                &job.facts.repository.full_name,
            )
            .await
            .internal()?;
        Ok(CacheMetadataResponse { entries })
    }
}

#[cfg(test)]
#[path = "../_tests_/cache_metadata_tests.rs"]
mod cache_metadata_tests;
