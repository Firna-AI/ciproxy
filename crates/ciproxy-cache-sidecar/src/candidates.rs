//! Assemble trusted local and upstream facts for the shared restore resolver.

use ciproxy_protocol::cache_metadata::CacheEntryMetadata;
use ciproxy_protocol::jobs::JobScope;

use crate::machine::{MachineCache, Session};
use crate::resolver::{Candidate, OrderingMode, resolve};
use crate::scope::read_scopes;
use crate::storage::interface::{Entry, EntryOrigin, Lookup};

/// Restore candidates after the shared resolver applies trusted ordering.
pub(crate) struct Candidates {
    /// Selected local archive, if its bytes are valid.
    pub(crate) local: Option<Entry>,
    /// Current complete listing used to verify a later runtime result.
    pub(crate) metadata: Option<Vec<CacheEntryMetadata>>,
    /// Exact upstream key selected by the same resolver.
    pub(crate) upstream: Option<CacheEntryMetadata>,
    /// Whether local-only legacy retention remains available.
    pub(crate) legacy: bool,
}

impl MachineCache {
    /// Acquire current provider facts before selecting either content store.
    pub(crate) async fn candidates(
        &self,
        session: &Session,
        keys: Vec<String>,
        version: &str,
    ) -> Candidates {
        let lookup = Lookup {
            scopes: read_scopes(
                &session.registration.facts,
                &session.scope,
                &session.private_ref,
            ),
            keys,
            version: version.into(),
        };
        let mut entries = self.store.entries().await.unwrap_or_default();
        let (metadata, legacy, local_only) = match self.metadata(session).await {
            Ok(metadata) => {
                let legacy = metadata.is_none();
                (metadata, legacy, legacy)
            }
            Err(_) => {
                tracing::warn!("Trusted cache metadata unavailable; using GitHub fallback");
                (
                    None,
                    false,
                    matches!(session.scope, JobScope::Unknown { .. }),
                )
            }
        };
        if let Some(metadata) = &metadata {
            for entry in &mut entries {
                if entry.origin == EntryOrigin::AcceptedSave && entry.source.is_none() {
                    entry.source = metadata
                        .iter()
                        .find(|source| {
                            entry.upstream_id.as_ref() == Some(&source.id)
                                && source.key == entry.key
                                && source.version == entry.version
                                && source.git_ref.0 == entry.scope.reference
                        })
                        .cloned();
                    if let Some(source) = &entry.source
                        && self.store.annotate_source(entry, source).await.is_err()
                    {
                        tracing::warn!("Unable to retain acquired cache source metadata");
                    }
                }
            }
        }
        let selection = match metadata.as_ref() {
            Some(remote) => match resolve(
                entries.clone(),
                remote,
                &lookup,
                OrderingMode::TrustedSource,
            ) {
                Some(candidate) => Some(candidate),
                None if remote.iter().all(|entry| {
                    entry.version != version
                        || !lookup
                            .scopes
                            .iter()
                            .any(|scope| scope.reference == entry.git_ref.0)
                        || !lookup.keys.iter().any(|key| entry.key.starts_with(key))
                }) =>
                {
                    resolve(entries.clone(), &[], &lookup, OrderingMode::LocalVisibility)
                }
                None => None,
            },
            None if local_only => {
                resolve(entries.clone(), &[], &lookup, OrderingMode::LocalVisibility)
            }
            None => None,
        };
        let (local, upstream) = match selection {
            Some(Candidate::Local(entry)) => (Some(*entry), None),
            Some(Candidate::Upstream(entry)) => (None, Some(entry)),
            None => (None, None),
        };
        let local = match local {
            Some(entry) if self.store.verify(&entry).await.is_ok() => Some(entry),
            _ => None,
        };
        Candidates {
            local,
            metadata,
            upstream,
            legacy,
        }
    }
}
