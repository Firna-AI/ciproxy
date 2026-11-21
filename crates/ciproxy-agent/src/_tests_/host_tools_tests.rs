//! Downloads cannot publish either executable before all integrity checks pass.

use std::sync::{Arc, Mutex};

use tokio::sync;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::host_fixtures::{archive, release};
use crate::host_tools::{HostDownloadMock, ToolFilesMock, Tools, VerifiedTools};

#[tokio::test]
async fn verified_pair_is_published_once_and_a_repaired_install_reuses_it() {
    let saved = Arc::new(Mutex::new(Vec::new()));
    let published = saved.clone();
    let mock = Arc::new(Unimock::new((
        ToolFilesMock::valid.stub(|each| {
            each.call(matching!())
                .answers(&|_| Ok(false))
                .once()
                .then()
                .answers(&|_| Ok(true));
        }),
        HostDownloadMock::fetch
            .some_call(matching!())
            .returns(Ok(archive())),
        ToolFilesMock::publish
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, checked| {
                *published.lock().unwrap() = checked
                    .files()
                    .into_iter()
                    .map(|(name, bytes, mode)| (name.to_owned(), bytes.to_vec(), mode))
                    .collect();
                Ok(())
            })),
    )));
    let tools = VerifiedTools {
        download: mock.clone(),
        files: mock.clone(),
        release: release(),
        operation: sync::Mutex::new(()),
    };
    tools.prepare().await.unwrap();
    tools.prepare().await.unwrap();
    assert_eq!(
        *saved.lock().unwrap(),
        vec![
            (
                "firecracker".to_owned(),
                b"fixture firecracker".to_vec(),
                0o700
            ),
            ("jailer".to_owned(), b"fixture jailer".to_vec(), 0o700),
        ]
    );
}

#[tokio::test]
async fn corrupted_archive_or_either_executable_never_reaches_publication() {
    let supplied = Arc::new(Mutex::new(archive()));
    let downloaded = supplied.clone();
    let saved = Arc::new(Mutex::new(0));
    let published = saved.clone();
    let mock = Arc::new(Unimock::new((
        ToolFilesMock::valid
            .each_call(matching!())
            .answers(&|_| Ok(false)),
        HostDownloadMock::fetch
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(downloaded.lock().unwrap().clone()))),
        ToolFilesMock::publish
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                *published.lock().unwrap() += 1;
                Ok(())
            })),
    )));
    for (damaged, outcome) in [
        ("valid", "valid"),
        ("archive", "invalid"),
        ("firecracker", "invalid"),
        ("jailer", "invalid"),
    ] {
        let mut expected = release();
        let mut bytes = archive();
        match damaged {
            "archive" => bytes[4] ^= 1,
            "firecracker" => expected.firecracker = "00",
            "jailer" => expected.jailer = "00",
            _ => (),
        }
        *supplied.lock().unwrap() = bytes;
        let tools = VerifiedTools {
            download: mock.clone(),
            files: mock.clone(),
            release: expected,
            operation: sync::Mutex::new(()),
        };
        let actual = match tools.prepare().await {
            Ok(()) => "valid",
            Err(Error::Preparation {
                component: "Firecracker release integrity",
                ..
            }) => "invalid",
            _ => "unexpected",
        };
        assert_eq!(actual, outcome, "{damaged}");
        assert_eq!(
            *saved.lock().unwrap(),
            1,
            "{damaged} must never add a publication"
        );
    }
}
