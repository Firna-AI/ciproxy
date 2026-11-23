//! Manager selection follows os-release, and installs name only missing packages.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::host_requirements::{Failure, HOST, Manager, NETWORK, missing, names};
use crate::host_system::{HostOutput, HostSystemMock};

#[test]
fn manager_follows_the_id_and_then_each_id_like_entry() {
    for (release, expected) in [
        ("ID=debian\n", Some(Manager::Apt)),
        ("ID=ubuntu\n", Some(Manager::Apt)),
        (
            "ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n",
            Some(Manager::Apt),
        ),
        ("ID=fedora\n", Some(Manager::Dnf)),
        ("ID=\"rhel\"\n", Some(Manager::Dnf)),
        (
            "ID=\"centos\"\nID_LIKE=\"rhel fedora\"\n",
            Some(Manager::Dnf),
        ),
        ("ID=rocky\n", Some(Manager::Dnf)),
        ("ID=almalinux\n", Some(Manager::Dnf)),
        (
            "ID=\"opensuse-leap\"\nID_LIKE=\"suse opensuse\"\n",
            Some(Manager::Zypper),
        ),
        ("ID=opensuse-tumbleweed\n", Some(Manager::Zypper)),
        ("ID=sles\n", Some(Manager::Zypper)),
        ("ID=arch\n", Some(Manager::Pacman)),
        ("ID=manjaro\nID_LIKE=arch\n", Some(Manager::Pacman)),
        ("ID=alpine\n", None),
        ("ID_LIKE=debian\n", Some(Manager::Apt)),
        ("", None),
    ] {
        assert_eq!(Manager::detect(release.as_bytes()), expected, "{release}");
    }
}

type Calls = Arc<Mutex<Vec<(String, Vec<String>)>>>;

fn recording(code: i32) -> (Arc<Unimock>, Calls) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let recorded = calls.clone();
    let mock = Arc::new(Unimock::new(
        HostSystemMock::execute
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, program, args| {
                recorded
                    .lock()
                    .unwrap()
                    .push((program.to_owned(), args.to_vec()));
                Ok(HostOutput {
                    code,
                    stdout: Vec::new(),
                })
            })),
    ));
    (mock, calls)
}

#[tokio::test]
async fn each_manager_installs_each_missing_package_once() {
    for (manager, expected) in [
        (
            Manager::Apt,
            vec![
                "apt-get update",
                "apt-get install -y --no-upgrade xfsprogs mount util-linux",
            ],
        ),
        (Manager::Dnf, vec!["dnf install -y xfsprogs util-linux"]),
        (
            Manager::Zypper,
            vec!["zypper --non-interactive install --no-recommends xfsprogs util-linux"],
        ),
        (
            Manager::Pacman,
            vec!["pacman -S --needed --noconfirm xfsprogs util-linux"],
        ),
    ] {
        let (mock, calls) = recording(0);
        let wanted = [HOST[5], HOST[6], HOST[7], HOST[10], HOST[11]];
        manager.install(mock.as_ref(), &wanted).await.unwrap();
        let calls = calls
            .lock()
            .unwrap()
            .iter()
            .map(|(program, args)| format!("{program} {}", args.join(" ")))
            .collect::<Vec<_>>();
        assert_eq!(calls, expected, "{manager:?}");
    }
}

#[tokio::test]
async fn failed_steps_report_refresh_or_install() {
    let (mock, _) = recording(1);
    assert_eq!(
        Manager::Apt.install(mock.as_ref(), NETWORK).await,
        Err(Failure::Refresh)
    );
    let (mock, calls) = recording(1);
    assert_eq!(
        Manager::Pacman.install(mock.as_ref(), NETWORK).await,
        Err(Failure::Install)
    );
    assert_eq!(
        calls.lock().unwrap().len(),
        1,
        "pacman never refreshes alone"
    );
    assert!(Manager::Pacman.repair().contains("pacman -Syu"));
}

#[tokio::test]
async fn missing_checks_commands_and_any_ca_bundle() {
    let mock = Unimock::new((
        HostSystemMock::execute
            .each_call(matching!(_, _))
            .answers(&|_, _, args| {
                let tool = args.last().map(String::as_str).unwrap_or_default();
                Ok(HostOutput {
                    code: i32::from(matches!(tool, "nft" | "losetup")),
                    stdout: Vec::new(),
                })
            }),
        HostSystemMock::read
            .each_call(matching!(_))
            .answers(&|_, path| {
                if path == "/etc/ssl/ca-bundle.pem" {
                    Ok(b"certificates".to_vec())
                } else {
                    Err(Error::Disconnected)
                }
            }),
    ));
    let found = missing(&mock, HOST).await.unwrap();
    assert_eq!(names(&found), "nft, losetup");
}
