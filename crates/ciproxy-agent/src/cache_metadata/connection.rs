//! Derive the host-only metadata connection from the configured gateway origin.

use ciproxy_protocol::cache_metadata::{CACHE_METADATA_PATH, CacheMetadataSource};
use ciproxy_protocol::secret::Secret;
use reqwest::Url;

/// Keep the gateway origin and replace only its transport scheme and route.
pub fn connection(gateway: &str, credential: Secret) -> Option<CacheMetadataSource> {
    let mut url = Url::parse(gateway).ok()?;
    let scheme = match url.scheme() {
        "wss" => "https",
        "ws" => "http",
        _ => return None,
    };
    if url.set_scheme(scheme).is_err() {
        return None;
    }
    url.set_path(CACHE_METADATA_PATH);
    url.set_query(None);
    url.set_fragment(None);
    Some(CacheMetadataSource {
        url: Secret::new(url.as_str()),
        credential,
    })
}

#[cfg(test)]
#[path = "../_tests_/cache_metadata_tests.rs"]
mod cache_metadata_tests;
