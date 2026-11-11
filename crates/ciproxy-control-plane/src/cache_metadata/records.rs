//! Narrow workspace-scoped store reads used by hosted metadata authorization.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::own_runners::RoutingSnapshot;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::Store;
use ciproxy_store_interface::tokens::Token;
use uuid::Uuid;

use crate::error::{ErrorResultExt, Result};

/// Read authenticated token authority and coherent workspace placement facts.
#[cfg_attr(test, unimock::unimock(api = MetadataRecordsMock))]
#[async_trait]
pub(crate) trait MetadataRecords: Send + Sync {
    /// Find an unrevoked token at the trusted request time.
    async fn authenticate(&self, hash: TokenHash, at: DateTime<Utc>) -> Result<Option<Token>>;
    /// Read only the token-derived workspace.
    async fn snapshot(&self, workspace_id: Uuid) -> Result<RoutingSnapshot>;
}

/// Adapt blocking hosted store queries to the asynchronous metadata service.
pub(crate) struct StoredMetadataRecords {
    /// Existing hosted persistence boundary.
    pub(crate) store: Arc<dyn Store>,
}

#[async_trait]
impl MetadataRecords for StoredMetadataRecords {
    async fn authenticate(&self, hash: TokenHash, at: DateTime<Utc>) -> Result<Option<Token>> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.authenticate_token(&hash, at))
            .await
            .internal()?
            .internal()
    }

    async fn snapshot(&self, workspace_id: Uuid) -> Result<RoutingSnapshot> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.own_routing_snapshot(workspace_id))
            .await
            .internal()?
            .internal()
    }
}
