//! One branch/key/exact/newest resolver for local and trusted upstream candidates.

use ciproxy_protocol::cache_metadata::CacheEntryMetadata;

use crate::storage::interface::{Entry, EntryOrigin, Lookup};

/// The facts available for comparison; local counters never compete with timestamps.
pub(crate) enum OrderingMode {
    /// Rank known source identities across both stores.
    TrustedSource,
    /// Preserve local-only ordering when no upstream identity participates.
    LocalVisibility,
}

/// The selected content store and its immutable candidate identity.
pub(crate) enum Candidate {
    /// A retained machine archive.
    Local(Box<Entry>),
    /// A current GitHub entry.
    Upstream(CacheEntryMetadata),
}

impl Candidate {
    fn key(&self) -> &str {
        match self {
            Self::Local(entry) => &entry.key,
            Self::Upstream(entry) => &entry.key,
        }
    }

    fn reference(&self, mode: &OrderingMode) -> &str {
        match self {
            Self::Local(entry) => match mode {
                OrderingMode::TrustedSource => entry
                    .source
                    .as_ref()
                    .map_or(&entry.scope.reference, |source| &source.git_ref.0),
                OrderingMode::LocalVisibility => &entry.scope.reference,
            },
            Self::Upstream(entry) => &entry.git_ref.0,
        }
    }

    fn created_at(&self, mode: &OrderingMode) -> Option<u64> {
        match self {
            Self::Local(entry) => match mode {
                OrderingMode::TrustedSource => entry
                    .source
                    .as_ref()
                    .and_then(|source| source.created_at.parse().ok()),
                OrderingMode::LocalVisibility => Some(entry.created),
            },
            Self::Upstream(entry) => entry.created_at.parse().ok(),
        }
    }
}

/// Apply branch, requested key, exact match, and newest-prefix order once.
pub(crate) fn resolve(
    entries: Vec<Entry>,
    remote: &[CacheEntryMetadata],
    lookup: &Lookup,
    mode: OrderingMode,
) -> Option<Candidate> {
    let visible: Vec<_> = entries
        .into_iter()
        .filter(|entry| entry.version == lookup.version && lookup.scopes.contains(&entry.scope))
        .collect();
    let mut candidates: Vec<_> = visible
        .iter()
        .filter(|entry| {
            (matches!(mode, OrderingMode::LocalVisibility)
                || entry.origin == EntryOrigin::AcceptedSave
                || entry.source.is_some())
                && (entry.origin == EntryOrigin::AcceptedSave
                    || !visible.iter().any(|saved| {
                        saved.origin == EntryOrigin::AcceptedSave
                            && saved.scope == entry.scope
                            && saved.key == entry.key
                    }))
        })
        .cloned()
        .map(|entry| Candidate::Local(Box::new(entry)))
        .collect();
    candidates.extend(
        remote
            .iter()
            .filter(|entry| entry.version == lookup.version)
            .cloned()
            .map(Candidate::Upstream),
    );
    for scope in &lookup.scopes {
        for key in &lookup.keys {
            let eligible = |entry: &&Candidate| {
                entry.reference(&mode) == scope.reference && entry.key().starts_with(key)
            };
            let exact = candidates
                .iter()
                .filter(eligible)
                .filter(|entry| entry.key() == key);
            if let Some(entry) = exact.max_by_key(|entry| matches!(entry, Candidate::Local(_))) {
                return Some(owned(entry));
            }
            let prefix: Vec<_> = candidates.iter().filter(eligible).collect();
            if prefix.iter().any(|entry| entry.created_at(&mode).is_none()) {
                if prefix
                    .iter()
                    .any(|entry| matches!(entry, Candidate::Upstream(_)))
                {
                    return None;
                }
                return prefix
                    .into_iter()
                    .max_by_key(|entry| match entry {
                        Candidate::Local(entry) => entry.created,
                        Candidate::Upstream(_) => 0,
                    })
                    .map(owned);
            }
            if let Some(entry) = prefix.into_iter().max_by_key(|entry| {
                (
                    entry.created_at(&mode),
                    matches!(entry, Candidate::Local(_)),
                    match entry {
                        Candidate::Local(entry) => entry.created,
                        Candidate::Upstream(_) => 0,
                    },
                )
            }) {
                return Some(owned(entry));
            }
        }
    }
    None
}

fn owned(entry: &Candidate) -> Candidate {
    match entry {
        Candidate::Local(entry) => Candidate::Local(entry.clone()),
        Candidate::Upstream(entry) => Candidate::Upstream(entry.clone()),
    }
}
