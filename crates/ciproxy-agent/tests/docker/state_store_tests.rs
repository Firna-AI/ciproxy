//! The real state store preserves a complete identity across fresh instances.

use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::secret::Secret;

use crate::boundaries::Store;
use crate::fixtures::TemporaryLayout;
use crate::persistence::FileStore;
use crate::state::{Identity, MachineState, Mode};

#[test]
fn second_file_store_loads_the_saved_credential_and_endpoint() {
    let temporary = TemporaryLayout::new();
    let writer = FileStore {
        directory: temporary.layout.data.clone(),
    };
    writer
        .save(&MachineState {
            identity: Some(Identity {
                workspace_id: WorkspaceId("preview-workspace".to_owned()),
                machine_id: MachineId("preview-machine".to_owned()),
                credential: Secret::new("synthetic-machine-credential"),
                endpoint: Some("wss://preview.ciproxy.example:8443/agent/".parse().unwrap()),
            }),
            mode: Mode::Draining,
            ..MachineState::default()
        })
        .unwrap();
    let reader = FileStore {
        directory: temporary.layout.data.clone(),
    };
    let loaded = reader.load().unwrap();
    assert_eq!(
        loaded.identity,
        Some(Identity {
            workspace_id: WorkspaceId("preview-workspace".to_owned()),
            machine_id: MachineId("preview-machine".to_owned()),
            credential: Secret::new("synthetic-machine-credential"),
            endpoint: Some("wss://preview.ciproxy.example:8443/agent/".parse().unwrap()),
        })
    );
    assert_eq!(loaded.mode, Mode::Draining);
}
