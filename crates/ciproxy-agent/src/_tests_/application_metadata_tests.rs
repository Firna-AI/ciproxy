//! A relay takes its endpoint and credential from one retained identity.

use std::sync::Arc;

use ciproxy_agent::cache_metadata::error::Error;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::secret::Secret;
use unimock::Unimock;

use crate::application_endpoint::{ConfiguredAction, configure};
use crate::application_metadata::MetadataTarget;
use crate::cli::Action;
use crate::endpoint::CommandEndpoint;
use crate::fixtures::identity;

#[tokio::test]
async fn missing_identity_endpoint_keeps_the_relay_available_without_a_remote_client() {
    let mut identity = identity();
    identity.endpoint = None;
    let target = MetadataTarget::for_machine(Some(&identity));
    assert!(matches!(&target, MetadataTarget::Unavailable));
    let client = target.client().unwrap().unwrap();
    let answer = client
        .lookup(JobId("job-during-local-uninstall".to_owned()))
        .await;
    assert!(matches!(answer, Err(Error::Unavailable)));
}

#[test]
fn relay_during_install_keeps_the_identity_preview_endpoint() {
    let mut identity = identity();
    identity.endpoint = Some("wss://preview.ciproxy.example:8443/agent/".parse().unwrap());
    let settings = CommandEndpoint {
        files: Arc::new(Unimock::new(())),
    };
    let action = configure(
        Action::Install {
            token: Secret::new("synthetic-command-token"),
            agent_url: Some("wss://app.ciproxy.dev/agent/".parse().unwrap()),
        },
        &settings,
    )
    .unwrap();
    let ConfiguredAction::Install { endpoint, .. } = action else {
        panic!("install must retain command intent");
    };
    assert_eq!(endpoint.as_str(), "wss://app.ciproxy.dev/agent/");
    let target = MetadataTarget::for_machine(Some(&identity));
    let MetadataTarget::Hosted(source) = target else {
        panic!("repair must retain hosted metadata for running jobs");
    };
    assert_eq!(
        source.url.expose_secret(),
        "https://preview.ciproxy.example:8443/agent/cache-metadata"
    );
    assert_eq!(source.credential.expose_secret(), "machine-secret");
}
