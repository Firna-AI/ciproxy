//! Dashboard HTTP integration tests with real PostgreSQL persistence.

mod http;

#[path = "../../ciproxy-overflow/tests/tls/facts.rs"]
mod boat_facts;
#[path = "../../ciproxy-overflow/tests/tls/peer.rs"]
mod boat_peer;
#[path = "../../ciproxy-overflow/tests/tls/wire.rs"]
mod wire;
