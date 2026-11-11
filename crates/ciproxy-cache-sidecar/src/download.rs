//! Download tickets and local entry creation with separate source and visibility.

use std::sync::Arc;

use ciproxy_protocol::cache_metadata::CacheEntryMetadata;

use crate::machine::{MachineCache, Session, identifier, transfer_url};
use crate::scope::write_scope;
use crate::storage::capture::Captured;
use crate::storage::interface::{Entry, EntryOrigin};

/// A verified local archive or an upstream transfer whose provenance can be retained.
pub(crate) enum TicketSource {
    /// Local bytes selected by the common resolver.
    Local(Box<Entry>),
    /// Upstream bytes and their separately verified source identity.
    Upstream {
        /// Runtime-authorized download URL.
        url: String,
        /// Source identity bound to the runtime result, if available.
        source: Option<CacheEntryMetadata>,
        /// Whether publication has sufficient facts for the selected mode.
        retain: bool,
    },
}

/// A session-bound transfer ticket and its observed archive coverage.
pub(crate) struct Download {
    pub key: String,
    pub version: String,
    pub entry: Option<Entry>,
    pub remote: String,
    pub blob: Option<Arc<Captured>>,
    pub local: bool,
    pub counted: bool,
    pub ranges: Vec<(u64, u64)>,
    pub generation: u64,
    pub source: Option<CacheEntryMetadata>,
    pub retain: bool,
}

impl MachineCache {
    /// Bind an archive selected by the resolver to one random session ticket.
    pub(crate) async fn download_ticket(
        &self,
        session: &Session,
        key: String,
        version: String,
        target: TicketSource,
    ) -> String {
        let id = identifier();
        let (entry, remote, source, retain) = match target {
            TicketSource::Local(entry) => (Some(*entry), String::new(), None, true),
            TicketSource::Upstream {
                url,
                source,
                retain,
            } => (None, url, source, retain),
        };
        let local = entry.is_some();
        session.downloads.lock().await.insert(
            id.clone(),
            Arc::new(tokio::sync::Mutex::new(Download {
                key,
                version,
                entry,
                remote,
                blob: None,
                local,
                counted: false,
                ranges: Vec::new(),
                generation: self.store.generation(),
                source,
                retain,
            })),
        );
        transfer_url(session, &id)
    }

    /// Form local visibility from the checked writer independently of source facts.
    pub(crate) fn entry(
        &self,
        session: &Session,
        key: String,
        version: String,
        origin: EntryOrigin,
    ) -> Entry {
        Entry {
            id: String::new(),
            key,
            version,
            origin,
            source: None,
            upstream_id: None,
            scope: write_scope(
                &session.registration.facts,
                &session.scope,
                &session.private_ref,
            ),
            size: 0,
            digest: String::new(),
            parts: Vec::new(),
            created: self.clock.now(),
            accessed: self.clock.now(),
        }
    }
}
