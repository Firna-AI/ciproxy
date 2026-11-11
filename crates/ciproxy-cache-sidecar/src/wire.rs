//! Typed JSON contracts from the official Actions cache clients.

use axum::body::Body;
use axum::http::{Response, StatusCode};
use serde::{Deserialize, Serialize};

pub(crate) const V2: &str = "/twirp/github.actions.results.api.v1.CacheService/";

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CacheRequest {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub version: String,
    #[serde(default, alias = "restore_keys")]
    pub restore_keys: Vec<String>,
    #[serde(default, alias = "size_bytes")]
    pub size_bytes: String,
    #[serde(default)]
    pub size: u64,
}

/// Exact upstream restore request after the shared resolver selects its key.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LookupRequest {
    /// Selected complete cache key.
    pub key: String,
    /// Original archive version.
    pub version: String,
    /// No prefix fallback is needed after the resolver selected a complete key.
    pub restore_keys: Vec<String>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CacheResponse {
    #[serde(default)]
    pub ok: bool,
    #[serde(
        default,
        alias = "signed_upload_url",
        skip_serializing_if = "String::is_empty"
    )]
    pub signed_upload_url: String,
    #[serde(
        default,
        alias = "signed_download_url",
        skip_serializing_if = "String::is_empty"
    )]
    pub signed_download_url: String,
    #[serde(
        default,
        alias = "matched_key",
        skip_serializing_if = "String::is_empty"
    )]
    pub matched_key: String,
    #[serde(default, alias = "entry_id", skip_serializing_if = "String::is_empty")]
    pub entry_id: String,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegacyEntry {
    #[serde(default)]
    pub archive_location: String,
    #[serde(default)]
    pub cache_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Reservation {
    pub cache_id: u64,
}

pub(crate) fn json(value: &impl Serialize) -> Response<Body> {
    match serde_json::to_vec(value) {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            response.headers_mut().insert(
                "content-type",
                "application/json"
                    .parse()
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/json")),
            );
            response
        }
        Err(_) => status(StatusCode::NO_CONTENT),
    }
}

pub(crate) fn status(code: StatusCode) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = code;
    response
}

pub(crate) fn miss(path: &str) -> Response<Body> {
    if path.starts_with(V2) {
        json(&CacheResponse::default())
    } else {
        status(StatusCode::NO_CONTENT)
    }
}

pub(crate) fn cache_path(path: &str) -> bool {
    path.starts_with(V2) || path.starts_with("/_apis/artifactcache/")
}

pub(crate) fn save_path(path: &str, method: &axum::http::Method) -> bool {
    path.starts_with(V2) && !path.ends_with("GetCacheEntryDownloadURL")
        || path.starts_with("/_apis/artifactcache/") && *method != axum::http::Method::GET
}
