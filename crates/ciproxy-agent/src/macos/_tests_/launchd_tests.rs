//! launchd files keep the restart rules of the design and escape their text.

use super::launchd::{agent_plist, launchctl, loaded_ids, newsyslog_conf, vm_helper_plist};
use super::paths::{local_id_of_label, valid_local_id, vm_label, vm_plist};

const ID: &str = "0123456789abcdef0123456789abcdef";

#[test]
fn the_agent_restarts_only_after_a_failure() {
    let plist = agent_plist();
    assert!(plist.contains("<key>Label</key>\n  <string>dev.ciproxy.agent</string>"));
    assert!(plist.contains(
        "<array><string>/bin/sh</string><string>-c</string><string>if [ -f /etc/ciproxy-agent/environment ] &amp;&amp; [ -O /etc/ciproxy-agent/environment ] &amp;&amp; [ -z \"$(find /etc/ciproxy-agent/environment -perm +022)\" ]; then set -a; . /etc/ciproxy-agent/environment; set +a; fi; exec /usr/local/bin/ciproxy-agent run</string></array>"
    ));
    assert!(plist.contains("<dict><key>SuccessfulExit</key><false/></dict>"));
    assert!(plist.contains("<key>ThrottleInterval</key>\n  <integer>5</integer>"));
    assert!(
        plist.contains(
            "<key>StandardErrorPath</key>\n  <string>/var/log/ciproxy/agent.log</string>"
        )
    );
    assert!(plist.ends_with("</dict>\n</plist>\n"));
}

#[test]
fn a_vm_helper_runs_once_with_its_local_id_only() {
    let plist = vm_helper_plist(ID);
    assert!(plist.contains(&format!("<string>dev.ciproxy.vm.{ID}</string>")));
    assert!(plist.contains(&format!("<string>vm-run</string><string>{ID}</string>")));
    assert!(plist.contains("<key>KeepAlive</key>\n  <false/>"));
    assert!(plist.contains("<key>ExitTimeOut</key>\n  <integer>30</integer>"));
    assert!(!plist.contains("SuccessfulExit"));
}

#[test]
fn launchctl_targets_the_system_domain() {
    assert_eq!(
        launchctl::bootstrap_vm(ID),
        [
            "bootstrap",
            "system",
            &format!("/var/db/ciproxy/vms/{ID}.plist")
        ]
    );
    assert_eq!(
        launchctl::bootout_vm(ID),
        ["bootout", &format!("system/dev.ciproxy.vm.{ID}")]
    );
    assert_eq!(
        launchctl::restart_agent(),
        ["kickstart", "-k", "system/dev.ciproxy.agent"]
    );
    assert_eq!(
        launchctl::bootstrap_agent(),
        [
            "bootstrap",
            "system",
            "/Library/LaunchDaemons/dev.ciproxy.agent.plist"
        ]
    );
}

#[test]
fn recovery_owns_only_labels_with_a_valid_local_id() {
    assert!(valid_local_id(ID));
    assert_eq!(local_id_of_label(&vm_label(ID)), Some(ID));
    for label in [
        "dev.ciproxy.agent",
        "dev.ciproxy.vm.",
        "dev.ciproxy.vm.0123456789ABCDEF0123456789ABCDEF",
        "dev.ciproxy.vm.0123456789abcdef0123456789abcde",
        "dev.ciproxy.vm.../../etc/passwd/0123456789abcdef",
        "com.example.vm.0123456789abcdef0123456789abcdef",
    ] {
        assert_eq!(local_id_of_label(label), None, "{label}");
    }
}

#[test]
fn paths_stay_in_the_owned_folders() {
    assert_eq!(
        vm_plist(ID).to_str(),
        Some(format!("/var/db/ciproxy/vms/{ID}.plist").as_str())
    );
}

#[test]
fn logs_rotate_without_a_signal() {
    let conf = newsyslog_conf();
    assert!(conf.contains("/var/log/ciproxy/agent.log\troot:wheel\t600\t5\t10240\t*\tNJ\n"));
}

#[test]
fn the_loaded_list_gives_only_our_helpers() {
    let list = format!(
        "PID\tStatus\tLabel\n412\t0\tdev.ciproxy.vm.{ID}\n-\t0\tdev.ciproxy.agent\n77\t0\tcom.apple.Finder\n-\t1\tdev.ciproxy.vm.bad\n"
    );
    assert_eq!(loaded_ids(&list), [ID]);
}
