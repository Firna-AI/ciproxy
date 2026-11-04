//! Azure single-shot and block uploads plus legacy ranged PATCH requests.

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Response, Uri};
use serde::Deserialize;

use crate::error::{CacheError, Result, ResultExt};
use crate::machine::{MachineCache, Session};
use crate::transport::Outbound;

#[derive(Deserialize)]
struct BlockList {
    #[serde(rename = "$value", default)]
    blocks: Vec<Block>,
}

#[derive(Deserialize)]
enum Block {
    Latest(String),
    Uncommitted(String),
    Committed(String),
}

impl MachineCache {
    pub(crate) async fn upload_blob(
        &self,
        session: Arc<Session>,
        uri: Uri,
        mut headers: HeaderMap,
        body: Body,
    ) -> Result<Response<Body>> {
        let id = uri.path().rsplit('/').next().unwrap_or_default();
        let pending = session
            .uploads
            .lock()
            .await
            .get(id)
            .cloned()
            .ok_or(CacheError::Unavailable)?;
        let mut upload = pending.lock().await;
        let mut url = url::Url::parse(&upload.remote).internal()?;
        let query: Vec<_> = url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
            .into_owned()
            .collect();
        url.query_pairs_mut()
            .extend_pairs(query.iter().map(|(key, value)| (key, value)));
        let component = query
            .iter()
            .find(|(key, _)| key == "comp")
            .map(|(_, value)| value.as_str());
        headers.remove("x-ciproxy-session");
        headers.remove("authorization");
        if component == Some("blocklist") {
            let bytes = to_bytes(body, 1024 * 1024).await.internal()?;
            let blocks: BlockList = quick_xml::de::from_reader(bytes.as_ref()).internal()?;
            let response = self
                .network
                .send(Outbound {
                    method: Method::PUT,
                    url: url.to_string(),
                    headers,
                    body: Body::from(bytes),
                })
                .await?;
            if response.status().is_success() {
                upload.whole = None;
                upload.order = blocks
                    .blocks
                    .into_iter()
                    .map(|block| match block {
                        Block::Latest(id) | Block::Uncommitted(id) | Block::Committed(id) => id,
                    })
                    .collect();
            }
            return Ok(response);
        }
        if let Some((_, id)) = query.iter().find(|(key, _)| key == "blockid") {
            upload.blocks.remove(id);
        } else {
            upload.whole = None;
        }
        let expected = headers
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        let (body, capture) = self.store.capture(body, expected).await;
        let response = self
            .network
            .send(Outbound {
                method: Method::PUT,
                url: url.to_string(),
                headers,
                body,
            })
            .await?;
        if response.status().is_success() {
            if let Some(capture) = capture {
                let blob = capture.complete.lock().await.clone();
                if let Some((_, id)) = query.iter().find(|(key, _)| key == "blockid") {
                    if let Some(blob) = blob {
                        upload.blocks.insert(id.clone(), blob);
                    } else {
                        upload.blocks.remove(id);
                    }
                } else {
                    upload.whole = blob;
                }
            } else {
                upload.whole = None;
                upload.blocks.clear();
            }
        }
        Ok(response)
    }

    pub(crate) async fn upload_legacy(
        &self,
        session: Arc<Session>,
        uri: Uri,
        headers: HeaderMap,
        body: Body,
    ) -> Result<Response<Body>> {
        let id = uri.path().rsplit('/').next().unwrap_or_default();
        let pending = session
            .uploads
            .lock()
            .await
            .get(id)
            .cloned()
            .ok_or(CacheError::Unavailable)?;
        let mut upload = pending.lock().await;
        let range = headers
            .get("content-range")
            .and_then(|value| value.to_str().ok())
            .and_then(legacy_range);
        let expected = headers
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        let (body, capture) = self.store.capture(body, expected).await;
        let response = self
            .upstream(&session, Method::PATCH, &upload.remote, headers, body)
            .await?;
        if response.status().is_success()
            && let (Some((start, end)), Some(capture)) = (range, capture)
            && let Some(blob) = capture.complete.lock().await.clone()
            && Some(blob.size) == end.checked_sub(start).and_then(|size| size.checked_add(1))
        {
            upload.ranges.insert(start, blob);
        }
        Ok(response)
    }
}

fn legacy_range(value: &str) -> Option<(u64, u64)> {
    let value = value.strip_prefix("bytes ")?.split('/').next()?;
    let (start, end) = value.split_once('-')?;
    let (start, end) = (start.parse().ok()?, end.parse().ok()?);
    if start <= end {
        Some((start, end))
    } else {
        None
    }
}
