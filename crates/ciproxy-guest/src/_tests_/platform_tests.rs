//! The runner launch, the socket paths, and the account lookup per platform.

use std::path::Path;

use crate::platform::{Account, Platform};

fn runner() -> Account {
    Account {
        name: "runner".into(),
        uid: 501,
        gid: 20,
    }
}

fn command_line(platform: Platform) -> Vec<String> {
    let command =
        platform.runner_command(&runner(), Path::new("/usr/local/libexec/ciproxy-runner"));
    let command = command.as_std();
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| part.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn linux_runner_starts_through_runuser() {
    assert_eq!(
        command_line(Platform::Linux),
        [
            "/usr/sbin/runuser",
            "--user",
            "runner",
            "--",
            "/usr/local/libexec/ciproxy-runner"
        ]
    );
}

#[test]
fn mac_runner_starts_in_the_account_domain_with_only_runner_values() {
    assert_eq!(
        command_line(Platform::MacOs),
        [
            "/bin/launchctl",
            "asuser",
            "501",
            "/usr/bin/sudo",
            "--non-interactive",
            "--user=runner",
            "--set-home",
            "--preserve-env=ACTIONS_RUNNER_INPUT_JITCONFIG,CIPROXY_ASSIGNMENT_SOCKET",
            "--",
            "/usr/local/libexec/ciproxy-runner"
        ]
    );
}

#[test]
fn mac_guest_uses_its_own_paths_and_no_docker() {
    assert_eq!(
        Platform::MacOs.assignment_socket(),
        "/var/run/ciproxy/assignment.sock"
    );
    assert!(
        Platform::MacOs
            .assignment_socket()
            .starts_with(Platform::MacOs.assignment_directory())
    );
    assert_eq!(Platform::MacOs.proxy_listen(), "127.0.0.1:8230");
    assert!(!Platform::MacOs.docker());
    assert!(Platform::Linux.docker());
}

#[test]
fn account_lookup_reads_the_user_database() {
    let root = Account::lookup("root").unwrap();
    assert_eq!((root.name.as_str(), root.uid, root.gid), ("root", 0, 0));
    assert!(Account::lookup("ciproxy-missing-account").is_err());
}
