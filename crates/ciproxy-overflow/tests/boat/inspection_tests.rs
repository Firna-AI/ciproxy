//! Read-only launch-marker evidence through the production verified TLS adapter.

use ciproxy_overflow::error::Error;
use ciproxy_overflow::maintenance::LaunchState;
use sandbox_interface::ProviderRef;
use serde_json::json;
use uuid::Uuid;

use crate::tls::peer::step;
use crate::tls::server::Server;

#[tokio::test]
async fn launch_inspection_reads_the_marker_without_replaying_a_launch() {
    for (exit_code, expected) in [
        (0, Ok(LaunchState::Started)),
        (10, Ok(LaunchState::NotStarted)),
        (1, Err(true)),
        (2, Err(true)),
    ] {
        let server = Server::new(vec![step(
            "POST",
            "/api/v1/sandboxes/bx_23456789/commands",
            200,
            json!({"ok":true,"type":"command.finished","success":exit_code == 0,"exitCode":exit_code,"signal":null,"stdout":"","stderr":"","timedOut":false}),
        )]).await;
        let result = server
            .backend
            .maintenance()
            .inspect_launch(ProviderRef::new("bx_23456789"), Uuid::from_u128(11))
            .await;
        assert_eq!(
            result.map_err(|error| matches!(error, Error::ProviderUnavailable)),
            expected
        );
        server.assert_consumed();
        let captured = server.captured();
        let command = captured[0].body["command"].as_str().unwrap();
        let script = command
            .strip_prefix("sudo -n /usr/bin/python3 -I -S -c '")
            .unwrap()
            .strip_suffix("' '/run/ciproxy-overflow/0000000000000000000000000000000b'")
            .unwrap();
        assert!(script.contains("os.stat(sys.argv[1], follow_symlinks=False)"));
        assert!(!script.contains("mkdir"));
        assert!(!script.contains("boat-run.sh"));
    }
}
