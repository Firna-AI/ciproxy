//! Real root-agent relay custody across the private cache administration boundary.

use serde_json::Value;

use crate::support::scoped::Scoped;
use crate::support::{Fixture, facts};

#[tokio::test]
async fn root_relay_keeps_machine_credential_out_of_cache_admin_and_guest_messages() {
    let mut fixture = Fixture::new().await;
    let peer = Scoped::start(&mut fixture).await;
    peer.add("refs/heads/main", "root-relay", "through-root", 100)
        .await;
    let job = fixture
        .job("relay-custody", facts("refs/heads/feature"))
        .await;
    assert_eq!(job.contents("root-relay", "v1", &[]).await, "through-root");
    let payloads = fixture.admin_bodies.lock().await;
    let json: Vec<Value> = payloads
        .iter()
        .filter_map(|bytes| serde_json::from_slice(bytes).ok())
        .collect();
    assert!(json.iter().any(|value| value["enabled"] == true));
    assert!(
        json.iter()
            .any(|value| value["metadata"]["entries"][0]["key"] == "root-relay")
    );
    assert!(
        payloads
            .iter()
            .all(|bytes| !String::from_utf8_lossy(bytes).contains("synthetic-machine-credential"))
    );
    assert!(
        !serde_json::to_string(&job.registration)
            .unwrap()
            .contains("synthetic-machine-credential")
    );
}
