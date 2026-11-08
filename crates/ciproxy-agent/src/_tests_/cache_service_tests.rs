//! Resumed cache installation proves the running unit before migration or release IO.

use std::sync::{Arc, Mutex};

use cli_updater::UpgradeOutcome;
use unimock::{MockFn, Unimock, matching};

use crate::cache_install::{CacheInstallFilesMock, CacheReleaseMock, InstalledCache};
use crate::cache_network::GuardedCacheStart;
use crate::cache_service::{CacheService, ReleasedCache};
use crate::cache_unit::{CacheUnit, CacheUnitFilesMock, InstalledCacheUnit, UNIT_PATH};
use crate::error::{Error, ErrorResultExt};
use crate::process::CommandsMock;

#[derive(Clone, Copy, Debug)]
enum Proof {
    Missing,
    Stale,
    Unreadable,
    ZeroPid,
    MalformedPid,
    FailedPidRead,
    ProtectedStartFailed,
    Current,
}

impl Proof {
    fn has_pid(self) -> bool {
        matches!(
            self,
            Self::Missing
                | Self::Stale
                | Self::Unreadable
                | Self::ProtectedStartFailed
                | Self::Current
        )
    }
}

#[tokio::test]
async fn resumed_current_file_stops_unproved_listener_before_migration_and_failing_download() {
    for proof in [
        Proof::Missing,
        Proof::Stale,
        Proof::Unreadable,
        Proof::ZeroPid,
        Proof::MalformedPid,
        Proof::FailedPidRead,
        Proof::ProtectedStartFailed,
    ] {
        let events = Arc::new(Mutex::new(Vec::new()));
        let version = if matches!(proof, Proof::Missing) {
            "0.0.0"
        } else {
            env!("CARGO_PKG_VERSION")
        };
        let service = service(proof, version, events.clone());
        let error = service.install().await.unwrap_err();
        assert_eq!(
            download_failure(&error),
            Some(("https://release.example/cache.tar.gz".to_owned(), 503))
        );
        let mut expected = vec!["published", "reload", "inspect"];
        if proof.has_pid() {
            expected.push("proof");
        }
        expected.extend([
            "stop", "loader", "loader", "restart", "activate", "download",
        ]);
        assert_eq!(
            *events.lock().unwrap(),
            expected,
            "{proof:?}: stop and protected activation must precede migration and download"
        );
    }
}

#[tokio::test]
async fn current_running_fingerprint_keeps_hourly_repair_without_restart_or_download() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let service = service(Proof::Current, env!("CARGO_PKG_VERSION"), events.clone());
    service.install().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "published",
            "reload",
            "inspect",
            "proof",
            "loader",
            "loader",
            "activate"
        ]
    );
}

fn service(proof: Proof, version: &str, events: Arc<Mutex<Vec<&'static str>>>) -> ReleasedCache {
    let command_events = events.clone();
    let commands = Arc::new(Unimock::new(
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, program, args, _| {
                let action = match (program, args.first().map(String::as_str)) {
                    ("systemctl", Some("daemon-reload")) => "reload",
                    ("systemctl", Some("show")) => "inspect",
                    ("systemctl", Some("stop")) => "stop",
                    ("systemctl", Some("try-restart")) => "restart",
                    ("systemctl", Some("enable")) => "activate",
                    ("/bin/sh", _) => "loader",
                    _ => return Ok(Vec::new()),
                };
                command_events.lock().unwrap().push(action);
                if action == "activate" && matches!(proof, Proof::ProtectedStartFailed) {
                    return Err(Error::HostCommand {
                        program: "systemctl".to_owned(),
                    });
                }
                if action == "inspect" {
                    assert_eq!(
                        args,
                        &[
                            "show",
                            "--property=MainPID",
                            "--value",
                            "ciproxy-cache.service"
                        ]
                    );
                    return match proof {
                        Proof::ZeroPid => Ok(b"0\n".to_vec()),
                        Proof::MalformedPid => Ok(b"not-a-pid\n".to_vec()),
                        Proof::FailedPidRead => Err(Error::HostCommand {
                            program: "systemctl".to_owned(),
                        }),
                        _ => Ok(b"42\n".to_vec()),
                    };
                }
                Ok(Vec::new())
            })),
    ));
    let current = current_unit();
    let expected_fingerprint = current
        .lines()
        .find_map(|line| line.strip_prefix("Environment=CIPROXY_CACHE_UNIT_SHA256="))
        .unwrap()
        .to_owned();
    let published = events.clone();
    let unit_calls = (
        CacheUnitFilesMock::read
            .some_call(matching!((path) if *path == UNIT_PATH))
            .returns(Ok(Some(current))),
        CacheUnitFilesMock::write
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, path, _| {
                if path == UNIT_PATH {
                    published.lock().unwrap().push("published");
                }
                Ok(())
            })),
    );
    let unit_files = if proof.has_pid() {
        let observed = events.clone();
        Unimock::new((
            unit_calls,
            CacheUnitFilesMock::process_matches
                .some_call(matching!(42, _))
                .answers_arc(Arc::new(move |_, _, fingerprint| {
                    assert_eq!(fingerprint, expected_fingerprint);
                    observed.lock().unwrap().push("proof");
                    matches!(proof, Proof::Current)
                })),
        ))
    } else {
        Unimock::new(unit_calls)
    };
    let release = if matches!(proof, Proof::Current) {
        Unimock::new(())
    } else {
        let downloaded = events.clone();
        Unimock::new(
            CacheReleaseMock::upgrade
                .some_call(matching!(_))
                .answers_arc(Arc::new(move |_, request| {
                    assert!(
                        request.force,
                        "unproved activation must keep checked release repair forced"
                    );
                    downloaded.lock().unwrap().push("download");
                    Err::<UpgradeOutcome, _>(cli_updater::Error::ReleaseServerStatus {
                        url: "https://release.example/cache.tar.gz".to_owned(),
                        status: 503,
                    })
                    .context("install released machine cache")
                })),
        )
    };
    ReleasedCache {
        files: Arc::new(Unimock::new(
            CacheInstallFilesMock::prepare
                .some_call(matching!())
                .returns(Ok(InstalledCache {
                    binary: "/test/cache-sidecar".into(),
                    version: version.to_owned(),
                    binary_exists: true,
                })),
        )),
        release: Arc::new(release),
        starter: Arc::new(GuardedCacheStart {
            commands: commands.clone(),
        }),
        commands,
        unit: Arc::new(InstalledCacheUnit {
            files: Arc::new(unit_files),
        }),
    }
}

fn current_unit() -> String {
    let text = Arc::new(Mutex::new(String::new()));
    let captured = text.clone();
    InstalledCacheUnit {
        files: Arc::new(Unimock::new(
            CacheUnitFilesMock::write
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, path, bytes| {
                    if path == UNIT_PATH {
                        *captured.lock().unwrap() = String::from_utf8(bytes.to_vec()).unwrap();
                    }
                    Ok(())
                })),
        )),
    }
    .install()
    .unwrap();
    text.lock().unwrap().clone()
}

fn download_failure(error: &Error) -> Option<(String, u16)> {
    let Error::Internal(error) = error else {
        return None;
    };
    match error.source_ref().downcast_ref::<cli_updater::Error>()? {
        cli_updater::Error::ReleaseServerStatus { url, status } => Some((url.clone(), *status)),
        _ => None,
    }
}
