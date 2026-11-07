//! Shared real-store overflow composition with replaceable GitHub boundaries.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_control_plane::boat_authority::BoatRunnerAuthority;
use ciproxy_github::registrations::RunnerRegistrations;
use ciproxy_overflow::api::{Overflow, OverflowConfig};
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;

use super::overflow_ports::{Factory, Ports};
use super::support::Fixture;

pub(super) fn authority(
    fixture: &Fixture,
    ports: &Arc<Ports>,
    registrations: Arc<dyn RunnerRegistrations>,
) -> Arc<BoatRunnerAuthority> {
    Arc::new(BoatRunnerAuthority {
        store: fixture.store.clone(),
        registrations,
        root_sizing: RootSizing::default(),
        clock: ports.clone(),
    })
}

pub(super) fn service(
    fixture: &Fixture,
    ports: &Arc<Ports>,
    registrations: Arc<dyn RunnerRegistrations>,
) -> Arc<dyn Overflow> {
    Arc::new(OverflowService::new(
        OverflowConfig {
            deployment: "runtime-integration".to_owned(),
            shared_snapshot: "shared-image".to_owned(),
            root_sizing: RootSizing::default(),
            exit_report_url: Secret::new("https://app.ciproxy.example/hooks/boat/runner-exit"),
        },
        OverflowDependencies {
            store: fixture.store.clone(),
            cipher: ports.clone(),
            factory: Arc::new(Factory(ports.clone())),
            runners: authority(fixture, ports, registrations),
            clock: ports.clone(),
            tokens: ports.clone(),
        },
    ))
}
