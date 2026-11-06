//! The actual global inet rule precedes any host cache restart or enable.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use super::{CacheStart, GuardedCacheStart};
use crate::network::network_name;
use crate::process::CommandsMock;

#[tokio::test]
async fn owned_global_ingress_policy_precedes_cache_service_start_for_all_vm_links() {
    let policy = Arc::new(Mutex::new(Vec::new()));
    let captured = policy.clone();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let nft_calls = calls.clone();
    let restart_calls = calls.clone();
    let enable_calls = calls.clone();
    let commands=Arc::new(Unimock::new((
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="nft"&&args==&["-f","-"])).answers_arc(Arc::new(move|_,_,_,input|{*captured.lock().unwrap()=input.to_vec();nft_calls.lock().unwrap().push("protected");Ok(Vec::new())})),
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="systemctl"&&args==&["try-restart","ciproxy-cache.service"])).answers_arc(Arc::new(move|_,_,_,_|{restart_calls.lock().unwrap().push("restarted");Ok(Vec::new())})),
  CommandsMock::run.next_call(matching!((program,args,_)if *program=="systemctl"&&args==&["enable","--now","ciproxy-cache.service"])).answers_arc(Arc::new(move|_,_,_,_|{enable_calls.lock().unwrap().push("enabled");Ok(Vec::new())})),
 )));
    let starter = GuardedCacheStart { commands };
    starter.start(true).await.unwrap();
    let policy = String::from_utf8(policy.lock().unwrap().clone()).unwrap();
    assert!(policy.contains("add table inet ciproxy_cache"));
    assert!(policy.contains("iifname != \"lo\" iifname != \"cpx*\" drop;"));
    assert!(network_name(1).starts_with("cpx") && network_name(16382).starts_with("cpx"));
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["protected", "restarted", "enabled"]
    );
}
