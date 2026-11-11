//! Get current metadata through the root agent relay, without its credential.

use ciproxy_protocol::cache_metadata::CacheEntryMetadata;
use ciproxy_protocol::jobs::JobScope;

use crate::error::Result;
use crate::machine::{MachineCache, Session};
use crate::scope::read_scopes;

impl MachineCache {
    /// Await current identities from the root agent on the private admin channel.
    pub(crate) async fn metadata(
        &self,
        session: &Session,
    ) -> Result<Option<Vec<CacheEntryMetadata>>> {
        Ok(self
            .metadata_requests
            .request(session.registration.job_id.clone())
            .await?
            .map(|response| response.entries))
    }

    /// Retain only one unchanged eligible identity around the runtime lookup.
    pub(crate) async fn bind_source(
        &self,
        session: &Session,
        before: Option<&[CacheEntryMetadata]>,
        key: &str,
        version: &str,
        reference: Option<&str>,
    ) -> Option<CacheEntryMetadata> {
        if matches!(session.scope, JobScope::Unknown { .. }) {
            return None;
        }
        let before = before?;
        let after = self.metadata(session).await.ok()??;
        let scopes = read_scopes(
            &session.registration.facts,
            &session.scope,
            &session.private_ref,
        );
        let eligible = |entry: &&CacheEntryMetadata| {
            entry.key == key
                && entry.version == version
                && scopes
                    .iter()
                    .any(|scope| scope.reference == entry.git_ref.0)
        };
        let previous: Vec<_> = before.iter().filter(eligible).collect();
        let current: Vec<_> = after.iter().filter(eligible).collect();
        if previous.len() != 1 || current.len() != 1 || previous[0] != current[0] {
            return None;
        }
        let source = current[0];
        if reference.is_some_and(|reference| reference != source.git_ref.0) {
            return None;
        }
        Some(source.clone())
    }
}
