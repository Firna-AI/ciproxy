//! Trusted upstream destinations that cannot change authority through a path.

use axum::body::Body;
use axum::http::Request;

use crate::error::{Result, ResultExt};
use crate::session::Registration;

pub(crate) fn canonicalize(mut request: Request<Body>) -> Result<Request<Body>> {
    let mut url = url::Url::parse("http://cache.invalid/").internal()?;
    url.set_path(request.uri().path());
    url.set_query(request.uri().query());
    let path = url
        .path()
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    let normalized = match url.query() {
        Some(query) => format!("/{path}?{query}"),
        None => format!("/{path}"),
    };
    *request.uri_mut() = normalized.parse().internal()?;
    Ok(request)
}

pub(crate) fn destination(registration: &Registration, path: &str) -> Result<String> {
    if path.starts_with("/_apis/artifactcache/") {
        Ok(format!(
            "{}{}",
            registration.cache_url.expose_secret().trim_end_matches('/'),
            path
        ))
    } else {
        let mut url = url::Url::parse(registration.results_url.expose_secret()).internal()?;
        let (path, query) = match path.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (path, None),
        };
        url.set_path(path);
        url.set_query(query);
        Ok(url.to_string())
    }
}
