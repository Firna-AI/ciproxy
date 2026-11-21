//! The actual global inet rule precedes any host cache restart or enable.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use super::{CacheStart, GuardedCacheStart};
use crate::cache_unit::{CacheUnit, CacheUnitFilesMock, InstalledCacheUnit, POLICY_PATH};
use crate::network::network_name;
use crate::process::CommandsMock;

#[tokio::test]
async fn owned_global_ingress_policy_precedes_cache_service_start_for_all_vm_links() {
    let policy = Arc::new(Mutex::new(Vec::new()));
    let captured = policy.clone();
    InstalledCacheUnit {
        files: Arc::new(Unimock::new(
            CacheUnitFilesMock::write
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, path, input| {
                    if path == POLICY_PATH {
                        *captured.lock().unwrap() = input.to_vec();
                    }
                    Ok(())
                })),
        )),
    }
    .install()
    .unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let nft_calls = calls.clone();
    let restart_calls = calls.clone();
    let enable_calls = calls.clone();
    let commands=Arc::new(Unimock::new((
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="systemd-sysusers"&&args==&["-"])).returns(Ok(Vec::new())),
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="/bin/sh"&&args==&["/etc/ciproxy-cache-firewall"])).answers_arc(Arc::new(move|_,_,_,_|{nft_calls.lock().unwrap().push("protected");Ok(Vec::new())})),
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="systemctl"&&args==&["try-restart","ciproxy-cache.service"])).answers_arc(Arc::new(move|_,_,_,_|{restart_calls.lock().unwrap().push("restarted");Ok(Vec::new())})),
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="systemctl"&&args==&["enable","--now","ciproxy-cache.service"])).answers_arc(Arc::new(move|_,_,_,_|{enable_calls.lock().unwrap().push("enabled");Ok(Vec::new())})),
 )));
    let starter = GuardedCacheStart { commands };
    starter.start(true).await.unwrap();
    let policy = String::from_utf8(policy.lock().unwrap().clone()).unwrap();
    assert!(policy.contains("add table inet ciproxy_cache"));
    assert!(policy.contains("iifname != \"lo\" iifname != \"cpx*\" counter drop;"));
    assert!(network_name(1).starts_with("cpx") && network_name(16382).starts_with("cpx"));
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["protected", "restarted", "enabled"]
    );
}

#[tokio::test]
async fn cache_account_and_output_policy_precede_each_managed_start() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let captured = calls.clone();
    let starter = GuardedCacheStart {
        commands: Arc::new(Unimock::new(
            CommandsMock::run
                .each_call(matching!(_, _, _))
                .answers_arc(Arc::new(move |_, program, args, input| {
                    captured.lock().unwrap().push((
                        program.to_owned(),
                        args.to_vec(),
                        input.to_vec(),
                    ));
                    Ok(Vec::new())
                })),
        )),
    };
    starter.start(true).await.unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(
        calls.first().unwrap().0,
        "systemd-sysusers",
        "create the cache identity before loading its UID policy"
    );
    assert!(
        std::str::from_utf8(&calls[0].2)
            .unwrap()
            .contains("u ciproxy-cache -")
    );
    assert_eq!(calls[1].0, "/bin/sh");
    assert_eq!(calls[1].1, vec!["/etc/ciproxy-cache-firewall"]);
    assert_eq!(calls[2].1, vec!["try-restart", "ciproxy-cache.service"]);
    assert_eq!(calls[3].1, vec!["enable", "--now", "ciproxy-cache.service"]);
}
