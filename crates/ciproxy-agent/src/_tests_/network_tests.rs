//! Captured host inputs prove TAP ownership and the guest network policy.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::fixtures::vm;
use crate::network::{LinuxNetwork, Network};
use crate::process::CommandsMock;

#[tokio::test]
async fn guest_network_allows_only_public_egress_and_its_cache_port() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let commands = captured.clone();
    let mock = Unimock::new(
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, program, args, input| {
                commands.lock().unwrap().push((
                    program.to_owned(),
                    args.to_vec(),
                    String::from_utf8(input.to_vec()).unwrap(),
                ));
                Ok(Vec::new())
            })),
    );
    LinuxNetwork {
        commands: Arc::new(mock),
    }
    .create(&vm("runner", false))
    .await
    .unwrap();
    let captured = captured.lock().unwrap();
    assert!(captured.iter().any(|(program, args, _)| program == "ip"
        && args.windows(2).any(|pair| pair == ["user", "100001"])));
    let root = &captured
        .iter()
        .find(|(program, _, _)| program == "nft")
        .unwrap()
        .2;
    let namespace = &captured
        .iter()
        .find(|(program, args, _)| program == "ip" && args.iter().any(|arg| arg == "nft"))
        .unwrap()
        .2;
    assert!(root.contains("iifname \"cpx1\" ip daddr 169.254.0.5 tcp dport 7887 accept"));
    assert!(root.contains("iifname \"cpx1\" drop"));
    assert!(root.contains("oifname \"cpx1\" ct state != established,related drop"));
    assert!(namespace.contains("meta nfproto ipv6 drop"));
    assert!(namespace.contains("iifname \"tap0\" ip saddr != 172.30.0.2 drop"));
    for blocked in [
        "10.0.0.0/8",
        "172.16.0.0/12",
        "192.168.0.0/16",
        "169.254.0.0/16",
        "168.63.129.16/32",
    ] {
        assert!(namespace.contains(blocked), "{blocked}");
        assert!(root.contains(blocked), "{blocked}");
    }
    assert!(namespace.contains("iifname \"tap0\" oifname \"uplink\" accept"));
    assert!(
        namespace.find("tcp dport 7887 accept").unwrap() < namespace.find("ip daddr {").unwrap()
    );
    assert!(
        namespace.find("ip daddr {").unwrap()
            < namespace.find("oifname \"uplink\" accept").unwrap()
    );
}
