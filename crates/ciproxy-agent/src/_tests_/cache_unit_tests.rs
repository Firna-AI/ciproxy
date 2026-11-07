//! Every managed listener start applies the policy installed with its unit.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::cache_network::RULES;
use crate::cache_unit::{
    CacheUnit, CacheUnitFilesMock, FIREWALL_PATH, InstalledCacheUnit, POLICY_PATH, UNIT_PATH,
};

#[test]
fn installed_cache_unit_requires_the_written_policy_before_its_listener() {
    let writes = Arc::new(Mutex::new(Vec::new()));
    let written = writes.clone();
    let unit = InstalledCacheUnit {
        files: Arc::new(Unimock::new(
            CacheUnitFilesMock::write
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, path, contents| {
                    written
                        .lock()
                        .unwrap()
                        .push((path.to_owned(), contents.to_vec()));
                    Ok(())
                })),
        )),
    };
    unit.install().unwrap();
    let installed = writes.lock().unwrap();
    let unit = installed
        .iter()
        .find(|(path, _)| path == UNIT_PATH)
        .unwrap();
    let lines: Vec<_> = std::str::from_utf8(&unit.1).unwrap().lines().collect();
    let expected = "ExecStartPre=+/bin/sh /etc/ciproxy-cache-firewall start".to_owned();
    assert!(lines.contains(&expected.as_str()));
    let prepare = lines.iter().position(|line| *line == expected).unwrap();
    let listener = lines
        .iter()
        .position(|line| line.starts_with("ExecStart="))
        .unwrap();
    assert!(prepare < listener);
    assert_eq!(
        installed.first(),
        Some(&(POLICY_PATH.to_owned(), RULES.as_bytes().to_vec()))
    );
}

#[test]
fn installed_cache_policy_blocks_private_upstream_and_preserves_only_inbound_replies() {
    let installed = installed_files();
    let policy = contents(&installed, POLICY_PATH);
    assert!(
        policy.contains("type filter hook output"),
        "the cache has no output firewall"
    );
    assert!(policy.contains("meta skuid \"ciproxy-cache\" jump service"));
    assert!(
        policy.contains("ct direction reply ct state established tcp sport { 7887, 8229 } return")
    );
    assert!(policy.contains("fib daddr type { local, broadcast, multicast } counter drop"));
    assert!(policy.contains("meta nfproto ipv6 counter drop"));
    assert!(policy.contains("169.254.0.0/16"));
    assert!(policy.contains("168.63.129.16/32"));
    assert!(policy.contains("ip daddr @dns4 udp dport 53 return"));
    assert!(policy.contains("ip6 daddr @dns6 tcp dport 53 return"));
    assert!(!policy.contains("ct state established,related"));
}

#[test]
fn installed_cache_service_uses_a_private_identity_data_mount_and_credential() {
    let installed = installed_files();
    let unit = contents(&installed, UNIT_PATH);
    assert!(
        unit.lines().any(|line| line == "User=ciproxy-cache"),
        "the cache service still runs as root"
    );
    assert!(unit.lines().any(|line| line == "Group=ciproxy-cache"));
    assert!(unit.contains("BindPaths=/var/lib/ciproxy/area/cache/data:/run/ciproxy-cache/data"));
    assert!(unit.contains(
        "--cache-dir /run/ciproxy-cache/data --admin-token-file /run/ciproxy-cache-auth/admin-token"
    ));
    assert!(unit.contains("ExecStartPre=+/bin/sh /etc/ciproxy-cache-firewall start"));
    assert!(unit.contains("ExecStopPost=+/bin/rm -f /run/ciproxy-cache-auth/admin-token"));
    assert!(unit.contains("NoNewPrivileges=yes"));
    assert!(unit.lines().any(|line| line == "CapabilityBoundingSet="));
    let loader = contents(&installed, FIREWALL_PATH);
    assert!(loader.contains("done < /etc/resolv.conf >> \"$policy\""));
    assert!(loader.contains("/usr/sbin/nft -f \"$policy\""));
    assert!(loader.contains("chown -R -h ciproxy-cache:ciproxy-cache \"$data\""));
    assert!(loader.contains("if [ \"${1:-}\" = start ]; then"));
    assert!(loader.contains("install -d -o root -g ciproxy-cache -m 0710 /run/ciproxy-cache-auth"));
    assert!(loader.contains("install -T -o root -g ciproxy-cache -m 0640"));
}

fn installed_files() -> Vec<(String, Vec<u8>)> {
    let writes = Arc::new(Mutex::new(Vec::new()));
    let captured = writes.clone();
    InstalledCacheUnit {
        files: Arc::new(Unimock::new(
            CacheUnitFilesMock::write
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, path, bytes| {
                    captured
                        .lock()
                        .unwrap()
                        .push((path.to_owned(), bytes.to_vec()));
                    Ok(())
                })),
        )),
    }
    .install()
    .unwrap();
    writes.lock().unwrap().clone()
}

fn contents(installed: &[(String, Vec<u8>)], path: &str) -> String {
    String::from_utf8(
        installed
            .iter()
            .find(|(name, _)| name == path)
            .unwrap()
            .1
            .clone(),
    )
    .unwrap()
}
