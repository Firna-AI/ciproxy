//! Gateway registration data shared by the PostgreSQL contract cases.

use ciproxy_protocol::cache::{DockerDiskReport, MachineCacheReport};
use ciproxy_protocol::images::ImageReport;
use ciproxy_protocol::reports::{DrainState, HardwareReport, MachineReport};
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::gateway::{Contact, Registration};
use ciproxy_store_interface::secrets::TokenHash;
use uuid::Uuid;

use super::support::{at, hash};

pub(super) fn registration(workspace_id: Uuid, install_hash: TokenHash) -> Registration {
    Registration {
        workspace_id,
        install_hash,
        machine_id: Uuid::new_v4(),
        credential_id: Uuid::new_v4(),
        credential_hash: hash(),
        contact: Contact {
            session_id: Uuid::new_v4(),
            at: at(),
            report: MachineReport {
                hostname: "builder".into(),
                agent_version: "0.1.0".into(),
                hardware: HardwareReport {
                    processor: "x64".into(),
                    threads: 4,
                    memory_bytes: 16_000_000_000,
                    kvm: false,
                    disks: vec![],
                },
                capacity: Slots(0),
                slots: vec![],
                runners: vec![],
                drain: DrainState::Active,
                cache: MachineCacheReport {
                    used_bytes: 0,
                    capacity_bytes: 0,
                    restores_last_7_days: 0,
                    hits_last_7_days: 0,
                    unknown_jobs: vec![],
                    jobs: vec![],
                },
                docker_disks: DockerDiskReport {
                    repositories: 0,
                    used_bytes: 0,
                },
                images: ImageReport {
                    available: vec![],
                    newest: None,
                },
            },
        },
    }
}
