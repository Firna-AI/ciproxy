//! Removed startup checks local networks without a connection credential.

use std::sync::{Arc, Mutex};

use unimock::Unimock;

use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, identity};
use crate::network_fixtures::{Deletion, HostNetworks, commands, restored, runtime, saving};
use crate::startup::{AgentStartup, Startup, StartupState};
use crate::state::{MachineState, Mode};
use crate::terminal_completion::CompletionAction;

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
        let agent = Arc::new(agent);
        let startup = AgentStartup {
            agent: agent.clone(),
            preparation: mock.clone(),
            images: mock.clone(),
        };
        let result = startup.prepare(identity.clone(), true).await;
        assert!(matches!(result, Err(Error::HostCommand { program }) if program == "ip"));
        assert_eq!(restored(&saved).pending_networks, [1].into());
        assert_eq!(restored(&saved).identity, identity);
        assert_eq!(agent.state.lock().await.mode, Mode::Removed);
        host.lock().unwrap().deletion = Deletion::Succeed;
        assert!(matches!(
            startup.prepare(identity.clone(), true).await,
            Ok(StartupState::Complete)
        ));
        assert_eq!(restored(&saved).pending_networks, Default::default());
        assert_eq!(restored(&saved).identity, None);
        assert_eq!(host.lock().unwrap().namespaces, Default::default());
        assert!(
            agent
                .complete_removal(CompletionAction::Stop)
                .await
                .unwrap()
        );
    }
    let agent = controller(MachineState::default(), Unimock::new(()));
    assert!(
        !agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
}
