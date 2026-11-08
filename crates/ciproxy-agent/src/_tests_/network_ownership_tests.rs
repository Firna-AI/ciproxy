//! Real VM allocation and network cleanup use reliable host inventory.

use std::sync::{Arc, Mutex};

use unimock::Unimock;

use crate::boundaries::Vms;
use crate::error::Error;
use crate::fixtures::vm;
use crate::network::{LinuxNetwork, Network};
use crate::network_fixtures::{Deletion, HostNetworks, commands, runtime};

#[tokio::test]
async fn allocation_skips_host_resources_and_owned_pending_indices() {
    let host = Arc::new(Mutex::new(HostNetworks {
        namespaces: ["cpx1".to_owned()].into(),
        links: ["cpx2p".to_owned()].into(),
        tables: ["cpx3".to_owned()].into(),
        addresses: vec!["169.254.0.17".to_owned()],
        ..HostNetworks::default()
    }));
    let mock = Arc::new(Unimock::new(commands(host.clone())));
    let vms = runtime(mock.clone());
    let allocated = vms
        .allocate(vm("new", false).report, &[5, 6])
        .await
        .unwrap();
    assert_eq!(allocated.network_index, 7);
    host.lock().unwrap().namespace_failure = true;
    let result = vms.allocate(vm("unknown", false).report, &[]).await;
    assert!(
        matches!(result, Err(Error::HostCommand { program }) if program == "namespace-inspection")
    );
    host.lock().unwrap().namespace_failure = false;
    host.lock().unwrap().malformed_tables = true;
    let result = vms.allocate(vm("malformed", false).report, &[]).await;
    assert!(matches!(result, Err(Error::Internal(_))));
}

#[tokio::test]
async fn failed_deletion_preserves_cleanup_until_absence_is_verified() {
    let host = Arc::new(Mutex::new(HostNetworks {
        namespaces: ["cpx1".to_owned()].into(),
        links: ["cpx1p".to_owned(), "unrelated".to_owned()].into(),
        tables: ["cpx1".to_owned(), "unrelated".to_owned()].into(),
        addresses: vec!["169.254.0.5".to_owned()],
        deletion: Deletion::Fail,
        ..HostNetworks::default()
    }));
    let mock = Arc::new(Unimock::new(commands(host.clone())));
    let network = LinuxNetwork {
        commands: mock.clone(),
    };
    let result = network.remove(1).await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "ip"));
    assert_eq!(host.lock().unwrap().deletions.len(), 3);
    host.lock().unwrap().deletion = Deletion::FailAfterRemoval;
    network.remove(1).await.unwrap();
    network.remove(1).await.unwrap();
    let host = host.lock().unwrap();
    assert_eq!(host.deletions.len(), 6);
    assert_eq!(host.namespaces, Default::default());
    assert_eq!(host.links, ["unrelated".to_owned()].into());
    assert_eq!(host.tables, ["unrelated".to_owned()].into());
    assert_eq!(host.addresses, vec!["169.254.0.5"]);
}

#[tokio::test]
async fn forwarding_only_orphans_remain_owned_and_block_reuse_until_cleanup() {
    let host = Arc::new(Mutex::new(HostNetworks {
        forwarding: ["CPX1".to_owned(), "UNRELATED".to_owned()].into(),
        ..HostNetworks::default()
    }));
    let network = LinuxNetwork {
        commands: Arc::new(Unimock::new(commands(host.clone()))),
    };
    assert_eq!(network.owned().await.unwrap(), vec![1]);
    assert_eq!(network.allocate(&[]).await.unwrap(), 2);
    host.lock().unwrap().deletion = Deletion::Fail;
    assert!(network.remove(1).await.is_err());
    assert_eq!(network.owned().await.unwrap(), vec![1]);
    host.lock().unwrap().deletion = Deletion::Succeed;
    network.remove(1).await.unwrap();
    assert_eq!(network.owned().await.unwrap(), Vec::<u16>::new());
    assert_eq!(
        host.lock().unwrap().forwarding,
        ["UNRELATED".to_owned()].into()
    );
}
