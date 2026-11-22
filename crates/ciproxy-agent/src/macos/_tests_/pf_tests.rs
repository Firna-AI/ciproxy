//! The pf rules keep the order that the spike proved: passes, then blocks.

use std::net::Ipv4Addr;

use super::pf::{
    DEFAULT_BRIDGE, DEFAULT_GATEWAY, DEFAULT_MASK, VmNetwork, enable_token, enabled, pfctl, rules,
    same_rules,
};

fn network() -> VmNetwork {
    VmNetwork::new(DEFAULT_GATEWAY, DEFAULT_MASK, DEFAULT_BRIDGE).expect("default network")
}

#[test]
fn rules_without_the_machine_cache_give_the_internet_only() {
    assert_eq!(
        rules(&network(), false),
        "# ciproxy job VM isolation v1\n\
         pass in quick on bridge100 inet proto udp from any port 68 to { 255.255.255.255, 192.168.64.1 } port 67\n\
         pass in quick on bridge100 inet proto { udp tcp } from 192.168.64.0/24 to 192.168.64.1 port 53\n\
         block drop in quick on bridge100 inet6 all\n\
         block drop in quick inet from 192.168.64.0/24 to (self)\n\
         block drop in quick inet from 192.168.64.0/24 to 192.168.64.0/24\n\
         block drop in quick inet from 192.168.64.0/24 to { 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, 100.64.0.0/10, 127.0.0.0/8, 224.0.0.0/4, 240.0.0.0/4 }\n\
         pass in quick on lo0 proto tcp from any to any port 7887\n\
         block drop in quick proto tcp from any to any port 7887\n"
    );
}

#[test]
fn the_machine_cache_pass_comes_before_every_block() {
    let text = rules(&network(), true);
    let cache = text
        .find("pass in quick on bridge100 inet proto tcp from 192.168.64.0/24 to 192.168.64.1 port 7887\n")
        .expect("cache pass");
    let first_block = text.find("block").expect("block");
    assert!(cache < first_block);
}

#[test]
fn the_network_refuses_odd_masks_and_other_interfaces() {
    let custom = VmNetwork::new(
        Ipv4Addr::new(10, 77, 8, 1),
        Ipv4Addr::new(255, 255, 252, 0),
        "bridge101",
    )
    .expect("custom network");
    assert_eq!(custom.subnet(), "10.77.8.0/22");
    for (mask, bridge) in [
        (Ipv4Addr::new(255, 0, 255, 0), "bridge100"),
        (Ipv4Addr::new(255, 255, 255, 255), "bridge100"),
        (Ipv4Addr::new(0, 0, 0, 0), "bridge100"),
        (DEFAULT_MASK, "en0"),
        (DEFAULT_MASK, "bridge"),
        (DEFAULT_MASK, "bridge100; pass all"),
    ] {
        assert_eq!(
            VmNetwork::new(DEFAULT_GATEWAY, mask, bridge),
            None,
            "{mask} {bridge}"
        );
    }
}

#[test]
fn pfctl_reads_the_token_and_the_status() {
    assert_eq!(
        enable_token("No ALTQ support in kernel\npf enabled\nToken : 18446742974197924607\n"),
        Some(18_446_742_974_197_924_607)
    );
    assert_eq!(enable_token("pf enabled\n"), None);
    assert!(enabled(
        "Status: Enabled for 0 days 00:01:02           Debug: Urgent\n"
    ));
    assert!(!enabled(
        "Status: Disabled                             Debug: Urgent\n"
    ));
    assert_eq!(pfctl::release(7), ["-X", "7"]);
    assert_eq!(pfctl::load(), ["-a", "com.apple/ciproxy", "-f", "-"]);
}

#[test]
fn loaded_rules_match_only_the_snapshot_after_the_load() {
    let snapshot =
        "pass in quick on bridge100 inet proto udp\nblock drop in quick on bridge100 inet6 all\n";
    assert!(same_rules(snapshot, &format!("\n{snapshot}  \n")));
    assert!(!same_rules(
        snapshot,
        "pass in quick on bridge100 inet proto udp\n"
    ));
    assert!(!same_rules("", ""));
}

#[test]
fn only_guests_inside_the_ruled_subnet_pass_the_check() {
    let network = network();
    assert!(network.contains(Ipv4Addr::new(192, 168, 64, 2)));
    assert!(network.contains(Ipv4Addr::new(192, 168, 64, 254)));
    assert!(!network.contains(Ipv4Addr::new(192, 168, 65, 2)));
    assert!(!network.contains(Ipv4Addr::new(10, 0, 0, 2)));
}
