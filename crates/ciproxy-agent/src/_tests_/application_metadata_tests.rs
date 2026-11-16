//! A retained identity without its route still gets explicit metadata fallback.

use ciproxy_agent::cache_metadata::error::Error;
use ciproxy_protocol::identifiers::JobId;

use crate::application_metadata::MetadataTarget;
use crate::fixtures::identity;

#[tokio::test]
async fn missing_saved_endpoint_keeps_the_relay_available_without_a_remote_client() {
    let identity = identity();
    let target = MetadataTarget::for_machine(None, Some(&identity));
    assert!(matches!(&target, MetadataTarget::Unavailable));
    let client = target.client().unwrap().unwrap();
    let answer = client
        .lookup(JobId("job-during-local-uninstall".to_owned()))
        .await;
    assert!(matches!(answer, Err(Error::Unavailable)));
}
