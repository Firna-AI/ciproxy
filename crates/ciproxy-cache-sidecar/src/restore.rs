//! Resolve both HTTP cache protocols through the same trusted candidate order.

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Response, Uri};
use bytes::Bytes;

use crate::download::TicketSource;
use crate::error::{Result, ResultExt};
use crate::machine::{MachineCache, Session};
use crate::wire::{CacheRequest, CacheResponse, LegacyEntry, LookupRequest, V2, json, miss};

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
        let candidates = self.candidates(&session, keys, &query.version).await;
        if let Some(entry) = candidates.local {
            let key = entry.key.clone();
            let url = self
                .download_ticket(
                    &session,
                    key.clone(),
                    query.version,
                    TicketSource::Local(Box::new(entry)),
                )
                .await;
            return Ok(json(&CacheResponse {
                ok: true,
                matched_key: key,
                signed_download_url: url,
                ..Default::default()
            }));
        }
        let mut headers = headers;
        let body = match &candidates.upstream {
            Some(entry) => {
                headers.remove("content-length");
                Body::from(
                    serde_json::to_vec(&LookupRequest {
                        key: entry.key.clone(),
                        version: query.version.clone(),
                        restore_keys: Vec::new(),
                    })
                    .internal()?,
                )
            }
            None => Body::from(bytes.clone()),
        };
        let response = self
            .upstream(
                &session,
                Method::POST,
                &format!("{V2}GetCacheEntryDownloadURL"),
                headers,
                body,
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
            let source = self
                .bind_source(
                    &session,
                    candidates.metadata.as_deref(),
                    &response.matched_key,
                    &query.version,
                    None,
                )
                .await;
            let retain = source.is_some() || candidates.legacy;
            response.signed_download_url = self
                .download_ticket(
                    &session,
                    response.matched_key.clone(),
                    query.version,
                    TicketSource::Upstream {
                        url: response.signed_download_url,
                        source,
                        retain,
                    },
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
        let candidates = self.candidates(&session, keys, &version).await;
        if let Some(entry) = candidates.local {
            let key = entry.key.clone();
            let archive_location = self
                .download_ticket(
                    &session,
                    key.clone(),
                    version,
                    TicketSource::Local(Box::new(entry)),
                )
                .await;
            return Ok(json(&LegacyEntry {
                archive_location,
                cache_key: key,
                scope: None,
            }));
        }
        let narrowed = candidates.upstream.as_ref().map(|entry| {
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("keys", &entry.key)
                .append_pair("version", &version)
                .finish();
            format!("{}?{query}", uri.path())
        });
        let response = self
            .upstream(
                &session,
                Method::GET,
                narrowed.as_deref().unwrap_or(path),
                headers,
                Body::empty(),
            )
            .await?;
        if !response.status().is_success() || response.status() == 204 {
            return Ok(response);
        }
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .internal()?;
        let mut response: LegacyEntry = serde_json::from_slice(&bytes).internal()?;
        if !response.archive_location.is_empty() {
            let source = self
                .bind_source(
                    &session,
                    candidates.metadata.as_deref(),
                    &response.cache_key,
                    &version,
                    response.scope.as_deref(),
                )
                .await;
            let retain = source.is_some() || candidates.legacy;
            response.archive_location = self
                .download_ticket(
                    &session,
                    response.cache_key.clone(),
                    version,
                    TicketSource::Upstream {
                        url: response.archive_location,
                        source,
                        retain,
                    },
                )
                .await;
        }
        Ok(json(&response))
    }
}
