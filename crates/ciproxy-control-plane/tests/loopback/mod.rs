//! Loopback test inputs that no other socket and no host proxy can change.

use std::time::Duration;

use ciproxy_control_plane::image_release::release_client;
use reqwest::ClientBuilder;

/// The production release client policy, with no proxy for local fixtures.
pub(crate) fn client() -> ClientBuilder {
    release_client(Duration::from_secs(30)).no_proxy()
}
