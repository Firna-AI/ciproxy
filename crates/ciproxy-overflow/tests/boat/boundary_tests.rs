//! The factory uses the pinned adapter and one credentialed wallet for every seam.

use std::{fs, os::unix::fs::PermissionsExt, process::Command};

use chrono::DateTime;
use rust_decimal::Decimal;
use sandbox_interface::{SandboxStartRefusal, SandboxStartRetry, TemporaryStartRefusalReason};
use serde_json::json;
use uuid::Uuid;

use crate::tls::facts::{identity, limits};
use crate::tls::server::{Server, step};

use super::support::request;

#[tokio::test]
async fn real_adapter_preserves_marks_snapshot_ttl_wallet_and_private_root_launch() {
    let server = Server::new(vec![
        step("GET", "/api/v1/me", 200, identity()),
        step("GET", "/api/v1/limits", 200, limits(false)),
        step("GET", "/api/v1/limits", 200, limits(false)),
        step("POST", "/api/v1/sandboxes", 202, json!({"ok":true,"type":"sandbox.created","status":"ready","ttlSeconds":21600,"sandbox":{"id":"bx_23456789","state":"ready","type":"large","ttlSeconds":21600}})),
        step("POST", "/api/v1/sandboxes/bx_23456789/commands", 200, command_finished()),
        step("POST", "/api/v1/sandboxes/bx_23456789/commands", 200, command_finished()),
        step("GET", "/api/v1/sandboxes/bx_23456789/usage?", 200, json!({"ok":true,"type":"sandbox.usage","sandboxId":"bx_23456789","since":"2026-11-05T12:00:00Z","until":"2026-11-05T12:01:30Z","dollars":0.0018,"running":true})),
        step("DELETE", "/api/v1/sandboxes/bx_23456789", 200, json!({"ok":true,"type":"deletion.operation","operation":{"id":"bdop_00000000000000000000000000000000","kind":"sandbox","targetId":"bx_23456789","status":"completed"}})),
    ]).await;
    let account = server.backend.account.account().await.unwrap();
    assert_eq!(account.name, "Customer Boat");
    assert_eq!(account.start_fee_bound_usd, Some(Decimal::new(7, 2)));
    let sizes = server.backend.capacity.account_capacity().await.unwrap();
    assert_eq!(sizes[0].resources.disk.amount, 111);
    let sandbox = server
        .backend
        .lifecycle
        .create_sandbox(request())
        .await
        .unwrap();
    server
        .backend
        .launcher
        .prepare(sandbox.provider_ref.clone(), Uuid::from_u128(11))
        .await
        .unwrap();
    server
        .backend
        .launcher
        .launch(sandbox.provider_ref.clone(), Uuid::from_u128(11))
        .await
        .unwrap();
    let until = DateTime::parse_from_rfc3339("2026-11-05T12:01:30Z")
        .unwrap()
        .to_utc();
    assert_eq!(
        server
            .backend
            .account
            .usage(sandbox.provider_ref.clone(), None, until)
            .await
            .unwrap()
            .unwrap()
            .usd,
        Decimal::new(18, 4)
    );
    server
        .backend
        .lifecycle
        .destroy_sandbox(sandbox.provider_ref)
        .await
        .unwrap();
    server.assert_consumed();
    let captured = server.captured();
    let create = &captured[3];
    assert_eq!(
        create.headers["idempotency-key"],
        "00000000-0000-0000-0000-00000000000b"
    );
    assert_eq!(create.body["noEnv"], true);
    assert_eq!(create.body["snapshots"], false);
    assert_eq!(create.body["monitoring"], false);
    assert_eq!(create.body["from"], "shared-image-v7");
    assert_eq!(create.body["ttlSeconds"], 21600);
    assert_eq!(
        create.body["labels"]["sandbox.workspace"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(create.body["labels"]["sandbox.backend"], "boat");
    verify_private_preparation(captured[4].body["command"].as_str().unwrap());
    let launch = captured[5].body["command"].as_str().unwrap();
    assert!(launch.contains("if mkdir /run/ciproxy-overflow/0000000000000000000000000000000b"));
    assert!(launch.contains("/opt/ciproxy/rootfs/opt/ciproxy-image/boat-run.sh"));
    assert!(!launch.contains("fixture-key"));
    assert_eq!(captured[7].headers["x-ascii-confirm-delete"], "bx_23456789");
}

#[tokio::test]
async fn advisory_capacity_does_not_hide_a_typed_create_refusal() {
    let server = Server::new(vec![
        step("GET", "/api/v1/limits", 200, limits(true)),
        step("POST", "/api/v1/sandboxes", 429, json!({"ok":false,"type":"sandbox.error","status":429,"code":"rate_limited","retryAfterSeconds":45,"nextStartAt":"2026-11-05T12:02:00Z"})),
    ]).await;
    assert_eq!(
        server
            .backend
            .capacity
            .account_capacity()
            .await
            .unwrap()
            .len(),
        1
    );
    let result = server.backend.lifecycle.create_sandbox(request()).await;
    let Err(sandbox_interface::Error::SandboxStartRefused { refusal, .. }) = result else {
        panic!("typed refusal required");
    };
    assert_eq!(
        refusal,
        SandboxStartRefusal::Temporary {
            reason: TemporaryStartRefusalReason::RateLimited,
            retry: SandboxStartRetry {
                retry_after: Some(std::time::Duration::from_secs(45)),
                next_start_at: Some("2026-11-05T12:02:00Z".parse().unwrap())
            }
        }
    );
    server.assert_consumed();
}

#[tokio::test]
async fn ambiguous_http_create_recovers_without_repeating_its_mutation() {
    let server = Server::new(vec![
        step("POST", "/api/v1/sandboxes", 0, json!(null)),
        step(
            "GET",
            "/api/v1/sandboxes?",
            200,
            json!({"ok":true,"type":"sandbox.list","sandboxes":[]}),
        ),
    ])
    .await;
    assert!(matches!(
        server.backend.lifecycle.create_sandbox(request()).await,
        Err(sandbox_interface::Error::BackendUnavailable { .. })
    ));
    assert_eq!(
        server
            .backend
            .lifecycle
            .recover_sandbox_create(request())
            .await
            .unwrap(),
        None
    );
    server.assert_consumed();
    let captured = server.captured();
    assert_eq!(
        captured
            .iter()
            .filter(|request| request.method == "POST")
            .count(),
        1
    );
    assert!(captured[1].path.contains("sandbox.operation"));
}

#[tokio::test]
async fn authorization_rejection_has_safe_diagnostics() {
    let server = Server::new(vec![step(
        "GET",
        "/api/v1/me",
        403,
        json!({"ok":false,"message":"fixture-key was rejected"}),
    )])
    .await;
    let result = server.backend.account.account().await;
    assert!(matches!(
        result,
        Err(ciproxy_overflow::error::Error::RejectedKey)
    ));
    assert!(!format!("{result:?}").contains("fixture-key"));
    server.assert_consumed();
}

fn command_finished() -> serde_json::Value {
    json!({"ok":true,"type":"command.finished","success":true,"exitCode":0,"signal":null,"stdout":"","stderr":"","timedOut":false})
}

fn verify_private_preparation(command: &str) {
    let script = command
        .strip_prefix("sudo -n /usr/bin/python3 -I -S -c '")
        .unwrap()
        .split_once("' '")
        .unwrap()
        .0;
    let account_fixture = "import os, pwd, types\npwd.getpwnam = lambda name: types.SimpleNamespace(pw_uid=os.getuid(), pw_gid=os.getgid())\n";
    let script = format!("{account_fixture}{script}");
    let temporary = tempfile::tempdir().unwrap();
    let directory = temporary.path().join("fresh-runtime");
    let registration = directory.join("registration.json");
    let run = || {
        let output = Command::new("/usr/bin/python3")
            .args(["-I", "-S", "-c", &script])
            .arg(&directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run();
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&registration).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::write(&registration, b"private-input-survives-prepare-retry").unwrap();
    run();
    assert_eq!(
        fs::read(registration).unwrap(),
        b"private-input-survives-prepare-retry"
    );
}
