//! Transfer tickets with observed byte coverage and safe signed-URL fallback.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Method, Response, StatusCode, Uri};
use futures_util::StreamExt;

use crate::download::Download;
use crate::error::{CacheError, Result, ResultExt};
use crate::machine::{MachineCache, Session};
use crate::storage::interface::{EntryOrigin, ReadSource};
use crate::transport::Outbound;
use crate::wire::status;

impl MachineCache {
    pub(crate) async fn blob(
        &self,
        session: Arc<Session>,
        method: Method,
        uri: Uri,
        headers: HeaderMap,
        body: Body,
    ) -> Result<Response<Body>> {
        if method == Method::PUT {
            return self.upload_blob(session, uri, headers, body).await;
        }
        if method != Method::GET && method != Method::HEAD {
            return Ok(status(StatusCode::METHOD_NOT_ALLOWED));
        }
        let id = uri.path().rsplit('/').next().unwrap_or_default();
        let ticket = session
            .downloads
            .lock()
            .await
            .get(id)
            .cloned()
            .ok_or(CacheError::Unavailable)?;
        let mut download = ticket.lock().await;
        if method == Method::HEAD && download.entry.is_none() && download.blob.is_none() {
            return self
                .network
                .send(Outbound {
                    method,
                    url: download.remote.clone(),
                    headers: HeaderMap::new(),
                    body: Body::empty(),
                })
                .await;
        }
        if download.entry.is_none()
            && download.blob.is_none()
            && self.fetch(&session, &mut download).await.is_err()
        {
            self.metrics.fallback(&session.registration.job_id).await;
            return redirect(&download.remote);
        }
        let (source, size) = if let Some(blob) = &download.blob {
            (ReadSource::Capture(blob.clone()), blob.size)
        } else if let Some(entry) = &download.entry {
            if method == Method::GET {
                let _ = self.store.touch(entry).await;
            }
            (ReadSource::Entry(Arc::new(entry.clone())), entry.size)
        } else {
            return Err(CacheError::Unavailable);
        };
        let (start, end, partial) = match range(&headers, size) {
            Some(range) => range,
            None => return Ok(status(StatusCode::RANGE_NOT_SATISFIABLE)),
        };
        let mut response = status(if partial {
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        });
        let length = end.saturating_sub(start);
        response
            .headers_mut()
            .insert("content-length", length.to_string().parse().internal()?);
        response
            .headers_mut()
            .insert("accept-ranges", "bytes".parse().internal()?);
        response.headers_mut().insert(
            "content-type",
            "application/octet-stream".parse().internal()?,
        );
        response
            .headers_mut()
            .insert("etag", "\"ciproxy-cache\"".parse().internal()?);
        if partial {
            response.headers_mut().insert(
                "content-range",
                format!("bytes {start}-{}/{size}", end - 1)
                    .parse()
                    .internal()?,
            );
        }
        if method == Method::HEAD {
            return Ok(response);
        }
        let local = download.local;
        let metrics = self.metrics.clone();
        let job = session.registration.job_id.clone();
        let ticket = ticket.clone();
        let store = self.store.clone();
        let stream = async_stream::stream! {
            let mut sent = 0;
            while sent < length {
                match store.read(&source, start + sent, (length - sent).min(65536) as usize).await {
                    Ok(chunk) if chunk.is_empty() => { metrics.fallback(&job).await; yield Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)); return; }
                    Ok(chunk) => {
                        sent += chunk.len() as u64;
                        if sent == length {
                            let mut download = ticket.lock().await;
                            if !download.counted {
                                download.ranges.push((start, end));
                            }
                            if !download.counted && covered(&mut download.ranges, size) {
                                download.counted = true;
                                metrics.restored(&job, local, size).await;
                            }
                        }
                        yield Ok::<_, std::io::Error>(chunk);
                    }
                    Err(_) => { metrics.fallback(&job).await; yield Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)); return; }
                }
            }
        };
        *response.body_mut() = Body::from_stream(stream);
        Ok(response)
    }

    async fn fetch(&self, session: &Session, download: &mut Download) -> Result<()> {
        let response = self
            .network
            .send(Outbound {
                method: Method::GET,
                url: download.remote.clone(),
                headers: HeaderMap::new(),
                body: Body::empty(),
            })
            .await?;
        if !response.status().is_success() {
            return Err(CacheError::Unavailable);
        }
        let expected = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok());
        let (body, capture) = self.store.capture(response.into_body(), expected).await;
        let capture = capture.ok_or(CacheError::Unavailable)?;
        let mut stream = body.into_data_stream();
        while let Some(chunk) = stream.next().await {
            chunk.internal()?;
        }
        let blob = capture
            .complete
            .lock()
            .await
            .clone()
            .ok_or(CacheError::Budget)?;
        if expected.is_some_and(|size| size != blob.size) {
            return Err(CacheError::Unavailable);
        }
        let entry = self.entry(
            session,
            download.key.clone(),
            download.version.clone(),
            EntryOrigin::ReaderFill,
        );
        if download.generation != self.store.generation() {
            return Err(CacheError::Unavailable);
        }
        match self.store.publish(entry, &blob).await {
            Ok(entry) => download.entry = Some(entry),
            Err(_) => download.blob = Some(blob),
        }
        Ok(())
    }
}

fn redirect(url: &str) -> Result<Response<Body>> {
    if url.is_empty() {
        return Ok(status(StatusCode::NO_CONTENT));
    }
    let mut response = status(StatusCode::TEMPORARY_REDIRECT);
    response
        .headers_mut()
        .insert("location", url.parse().internal()?);
    Ok(response)
}

fn range(headers: &HeaderMap, size: u64) -> Option<(u64, u64, bool)> {
    let Some(range) = headers.get("range").or_else(|| headers.get("x-ms-range")) else {
        return Some((0, size, false));
    };
    let (start, end) = range
        .to_str()
        .ok()?
        .strip_prefix("bytes=")?
        .split_once('-')?;
    if start.is_empty() {
        let length = end.parse::<u64>().ok()?;
        if size == 0 || length == 0 {
            return None;
        }
        return Some((size.saturating_sub(length), size, true));
    }
    let start = start.parse::<u64>().ok()?;
    let end = if end.is_empty() {
        size
    } else {
        end.parse::<u64>().ok()?.checked_add(1)?.min(size)
    };
    if start >= end {
        None
    } else {
        Some((start, end, true))
    }
}

fn covered(ranges: &mut Vec<(u64, u64)>, size: u64) -> bool {
    ranges.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (start, end) in ranges.iter().copied() {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    let complete = merged.len() == 1 && merged[0] == (0, size);
    *ranges = merged;
    complete
}
