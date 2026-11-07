//! Removed startup checks local networks without a connection credential.

use std::sync::{Arc, Mutex};

use unimock::Unimock;

use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, identity};
use crate::network_fixtures::{Deletion, HostNetworks, commands, restored, runtime, saving};
use crate::state::{MachineState, Mode};

#[tokio::test]
async fn removed_startup_retries_orphan_cleanup_without_requiring_identity() {
    for identity in [None, Some(identity())] {
        let host = Arc::new(Mutex::new(HostNetworks {
            namespaces: ["cpx1".to_owned()].into(),
            deletion: Deletion::Fail,
            ..HostNetworks::default()
        }));
        let saved = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Unimock::new((
            commands(host.clone()),
            saving(saved.clone()),
        )));
        let initial = MachineState {
            mode: Mode::Removed,
            identity: identity.clone(),
            ..MachineState::default()
        };
        let mut agent = controller(initial, mock.as_ref().clone());
        agent.vms = Arc::new(runtime(mock.clone()));
        let result = agent.finish_removed_startup().await;
        assert!(matches!(result, Err(Error::HostCommand { program }) if program == "ip"));
        assert_eq!(restored(&saved).pending_networks, [1].into());
        assert!(!agent.finished().await);
        host.lock().unwrap().deletion = Deletion::Succeed;
        assert!(agent.finish_removed_startup().await.unwrap());
        assert_eq!(restored(&saved).pending_networks, Default::default());
        assert_eq!(restored(&saved).identity, identity);
        assert_eq!(host.lock().unwrap().namespaces, Default::default());
        assert!(agent.finished().await);
    }
    let agent = controller(MachineState::default(), Unimock::new(()));
    assert!(!agent.finish_removed_startup().await.unwrap());
}
