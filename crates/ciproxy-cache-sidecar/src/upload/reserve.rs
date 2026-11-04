//! Forward cache reservations and bind returned upload locations to a session.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Response};
use bytes::Bytes;
use tokio::sync::Mutex;

use crate::error::{Result, ResultExt};
use crate::machine::{MachineCache, Session, identifier, transfer_url};
use crate::storage::interface::EntryOrigin;
use crate::wire::{CacheRequest, CacheResponse, Reservation, V2, json};

use super::state::{Archive, Upload};

impl MachineCache {
    pub(crate) async fn reserve(
        &self,
        session: Arc<Session>,
        path: &str,
        bytes: &Bytes,
        headers: HeaderMap,
    ) -> Result<Response<Body>> {
        let query: CacheRequest = serde_json::from_slice(bytes).internal()?;
        let response = self
            .upstream(
                &session,
                Method::POST,
                path,
                headers,
                Body::from(bytes.clone()),
            )
            .await?;
        if !response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .internal()?;
        let (id, remote, mut response) = if path.starts_with(V2) {
            let mut response: CacheResponse = serde_json::from_slice(&body).internal()?;
            if !response.ok {
                return Ok(json(&response));
            }
            let id = identifier();
            let remote = response.signed_upload_url;
            response.signed_upload_url = transfer_url(&session, &id);
            (id, remote, json(&response))
        } else {
            let reservation: Reservation = serde_json::from_slice(&body).internal()?;
            let id = reservation.cache_id.to_string();
            (
                id.clone(),
                format!("/_apis/artifactcache/caches/{id}"),
                json(&reservation),
            )
        };
        session.uploads.lock().await.insert(
            id,
            Arc::new(Mutex::new(Upload {
                key: query.key,
                version: query.version,
                remote,
                active: if path.starts_with(V2) {
                    Archive::Whole(None)
                } else {
                    Archive::Ranges(BTreeMap::new())
                },
                staged: BTreeMap::new(),
                generation: self.store.generation(),
            })),
        );
        *response.status_mut() = status;
        Ok(response)
    }

    pub(crate) async fn finalize(
        &self,
        session: Arc<Session>,
        path: &str,
        bytes: &Bytes,
        headers: HeaderMap,
    ) -> Result<Response<Body>> {
        let query: CacheRequest = serde_json::from_slice(bytes).internal()?;
        let pending = session.uploads.lock().await.clone();
        let upload = if path.starts_with(V2) {
            let mut found = None;
            for upload in pending.values() {
                let guard = upload.lock().await;
                if guard.key == query.key && guard.version == query.version {
                    found = Some(upload.clone());
                    break;
                }
            }
            found
        } else {
            path.rsplit('/')
                .next()
                .and_then(|id| pending.get(id))
                .cloned()
        };
        let response = self
            .upstream(
                &session,
                Method::POST,
                path,
                headers,
                Body::from(bytes.clone()),
            )
            .await?;
        if !response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .internal()?;
        let accepted = !path.starts_with(V2)
            || serde_json::from_slice::<CacheResponse>(&body)
                .internal()?
                .ok;
        if accepted && let Some(upload) = &upload {
            let upload = upload.lock().await;
            let size = if path.starts_with(V2) {
                query.size_bytes.parse::<u64>().ok()
            } else {
                Some(query.size)
            };
            if upload.generation == self.store.generation()
                && let Some(blob) = self.assemble(&upload).await
                && size == Some(blob.size)
            {
                let entry = self.entry(
                    &session,
                    upload.key.clone(),
                    upload.version.clone(),
                    EntryOrigin::AcceptedSave,
                );
                if self.store.publish(entry, &blob).await.is_err() {
                    tracing::warn!("Local cache save unavailable");
                }
            }
        }
        if let Some(upload) = upload {
            session
                .uploads
                .lock()
                .await
                .retain(|_, candidate| !Arc::ptr_eq(candidate, &upload));
        }
        let mut response = Response::new(Body::from(body));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        Ok(response)
    }
}
