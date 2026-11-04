//! Version rejection before any credential or persistent side effect.

use ciproxy_protocol::agent::AgentMessage;
use unimock::Unimock;

use crate::error::Error;

use super::support::gateway;

#[test]
fn rejects_unsupported_version_before_authentication() {
    let service = gateway(
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
        Unimock::new(()),
    );
    let message: AgentMessage =
        serde_json::from_str(include_str!("unsupported_hello.json")).unwrap();
    assert!(matches!(
        service.open(message),
        Err(Error::UnsupportedVersion)
    ));
}
