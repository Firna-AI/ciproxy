//! The production Boat capabilities use the shared local HTTPS peer.

use ciproxy_overflow::provider::{BackendFactory, Capabilities};
use ciproxy_protocol::secret::Secret;

use super::peer::{Peer, Step};
use super::wire::Request;

pub(crate) struct Server {
    pub(crate) backend: Capabilities,
    peer: Peer,
}

impl Server {
    pub(crate) async fn new(steps: Vec<Step>) -> Self {
        let peer = Peer::new(steps).await;
        let backend = peer
            .factory
            .backend(&Secret::new("fixture-key"), "personal")
            .unwrap();
        Self { backend, peer }
    }

    pub(crate) fn captured(&self) -> Vec<Request> {
        self.peer.captured()
    }

    pub(crate) fn assert_consumed(&self) {
        self.peer.assert_consumed();
    }
}
