//! Fixed machine data and gateway composition without ambient input.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::gateway::{GatewayStore, MachineSnapshot};
use ciproxy_store_interface::machines::Machine;
use unimock::Unimock;
use uuid::Uuid;

use crate::boundary::{AgentEvents, Authorization, Clock, Entropy};
use crate::connections::Connections;
use crate::gateway::{Gateway, MachineGateway};

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-04T12:00:00Z".parse().unwrap()
}
pub(super) fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

pub(super) fn snapshot() -> MachineSnapshot {
    MachineSnapshot {
        machine: Machine {
            id: id(3),
            hostname: "builder".into(),
            processor: "x64".into(),
            hardware_threads: 4,
            memory_bytes: 16_000_000_000,
            kvm_available: true,
            slots: 1,
            warm_runners: 2,
            draining: false,
            last_contact_at: Some(at()),
            agent_version: "0.1.0".into(),
            disks: vec![],
            removed_at: None,
        },
        report: None,
    }
}

pub(super) fn gateway(
    store: Unimock,
    authorization: Unimock,
    clock: Unimock,
    entropy: Unimock,
    connections: Unimock,
    events: Unimock,
) -> Arc<dyn MachineGateway> {
    let store: Arc<dyn GatewayStore> = Arc::new(store);
    let authorization: Arc<dyn Authorization> = Arc::new(authorization);
    let clock: Arc<dyn Clock> = Arc::new(clock);
    let entropy: Arc<dyn Entropy> = Arc::new(entropy);
    let connections: Arc<dyn Connections> = Arc::new(connections);
    let events: Arc<dyn AgentEvents> = Arc::new(events);
    Arc::new(Gateway::new(
        store,
        authorization,
        clock,
        entropy,
        connections,
        events,
    ))
}
