//! Host metadata follows the configured gateway origin with no guest input.

use ciproxy_protocol::secret::Secret;

use super::connection;

#[test]
fn metadata_connection_uses_gateway_origin_and_machine_credential() {
    let source = connection(
        "wss://gateway.example:8443/agent/socket?debug=1#ignored",
        Secret::new("synthetic-machine-credential"),
    )
    .unwrap();
    assert_eq!(
        source.url.expose_secret(),
        "https://gateway.example:8443/agent/cache-metadata"
    );
    assert_eq!(
        source.credential.expose_secret(),
        "synthetic-machine-credential"
    );
    assert!(!format!("{source:?}").contains("synthetic-machine-credential"));
}
