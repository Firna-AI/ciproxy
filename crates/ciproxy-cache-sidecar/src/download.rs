//! Local-first cache metadata lookup without trusting caller scope metadata.

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Response, Uri};
use bytes::Bytes;

use crate::error::{Result, ResultExt};
use crate::machine::{MachineCache, Session, identifier, transfer_url};
use crate::scope::{read_scopes, write_scope};
use crate::storage::capture::Captured;
use crate::storage::interface::{Entry, EntryOrigin, Lookup};
use crate::wire::{CacheRequest, CacheResponse, LegacyEntry, V2, json, miss};

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
}

impl MachineCache {
    pub(crate) async fn lookup_v2(
        &self,
        session: Arc<Session>,
        bytes: &Bytes,
        headers: HeaderMap,
    ) -> Result<Response<Body>> {
        let query: CacheRequest = serde_json::from_slice(bytes).internal()?;
        if !session.registration.permissions.read {
            return Ok(miss(V2));
        }
        let mut keys = vec![query.key.clone()];
        keys.extend(query.restore_keys);
        let local = self.local(&session, keys, &query.version).await;
        if let Some(entry) = local {
            let key = entry.key.clone();
            let url = self
                .download_ticket(
                    &session,
                    key.clone(),
                    query.version,
                    Some(entry),
                    String::new(),
                )
                .await;
            return Ok(json(&CacheResponse {
                ok: true,
                matched_key: key,
                signed_download_url: url,
                ..Default::default()
            }));
        }
        let response = self
            .upstream(
                &session,
                Method::POST,
                &format!("{V2}GetCacheEntryDownloadURL"),
                headers,
                Body::from(bytes.clone()),
            )
            .await?;
        if !response.status().is_success() {
            return Ok(response);
        }
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .internal()?;
        let mut response: CacheResponse = serde_json::from_slice(&bytes).internal()?;
        if response.ok && !response.signed_download_url.is_empty() {
            response.signed_download_url = self
                .download_ticket(
                    &session,
                    response.matched_key.clone(),
                    query.version,
                    None,
                    response.signed_download_url,
                )
                .await;
        }
        Ok(json(&response))
    }

    pub(crate) async fn lookup_legacy(
        &self,
        session: Arc<Session>,
        uri: Uri,
        headers: HeaderMap,
    ) -> Result<Response<Body>> {
        let path = uri
            .path_and_query()
            .map_or(uri.path(), |value| value.as_str());
        if !session.registration.permissions.read {
            return Ok(miss(path));
        }
        let query: Vec<_> = url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
            .into_owned()
            .collect();
        let keys = query
            .iter()
            .find(|(name, _)| name == "keys")
            .map_or_else(Vec::new, |(_, value)| {
                value.split(',').map(str::to_owned).collect()
            });
        let version = query
            .iter()
            .find(|(name, _)| name == "version")
            .map_or_else(String::new, |(_, value)| value.clone());
        if let Some(entry) = self.local(&session, keys, &version).await {
            let key = entry.key.clone();
            let archive_location = self
                .download_ticket(&session, key.clone(), version, Some(entry), String::new())
                .await;
            return Ok(json(&LegacyEntry {
                archive_location,
                cache_key: key,
            }));
        }
        let response = self
            .upstream(&session, Method::GET, path, headers, Body::empty())
            .await?;
        if !response.status().is_success() || response.status() == 204 {
            return Ok(response);
        }
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .internal()?;
        let mut response: LegacyEntry = serde_json::from_slice(&bytes).internal()?;
        if !response.archive_location.is_empty() {
            response.archive_location = self
                .download_ticket(
                    &session,
                    response.cache_key.clone(),
                    version,
                    None,
                    response.archive_location,
                )
                .await;
        }
        Ok(json(&response))
    }

    async fn local(&self, session: &Session, keys: Vec<String>, version: &str) -> Option<Entry> {
        let lookup = Lookup {
            scopes: read_scopes(&session.registration.facts, &session.registration.job_id),
            keys,
            version: version.to_owned(),
        };
        match self.store.lookup(&lookup).await {
            Ok(Some(entry)) if self.store.verify(&entry).await.is_ok() => Some(entry),
            Ok(_) => None,
            Err(_) => {
                tracing::warn!("Local cache lookup unavailable");
                None
            }
        }
    }

    async fn download_ticket(
        &self,
        session: &Session,
        key: String,
        version: String,
        entry: Option<Entry>,
        remote: String,
    ) -> String {
        let id = identifier();
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
            })),
        );
        transfer_url(session, &id)
    }

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
            scope: write_scope(&session.registration.facts, &session.registration.job_id),
            size: 0,
            digest: String::new(),
            parts: Vec::new(),
            created: self.clock.now(),
            accessed: self.clock.now(),
        }
    }
}
