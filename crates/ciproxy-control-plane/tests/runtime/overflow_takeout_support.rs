//! Production GitHub retirement adapters for real-store take-out cases.

use std::sync::Arc;

use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RegistrationService;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;

use super::overflow_support::Harness;
use super::recovery_support::other_organization;
use super::takeout_installations::Probe;
use super::takeout_support::{Factory, Remote};

pub(super) fn prepare(harness: &mut Harness, remote: Arc<Remote>) -> LifecycleEvent {
    let fixture = &harness.fixture;
    other_organization(fixture);
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    let event = fixture.store.pending_github_lifecycle(42).unwrap()[0]
        .event
        .clone();
    let registrations = Arc::new(RegistrationService {
        store: Arc::new(StoreAdapter::new(
            fixture.store.clone(),
            harness.ports.clone(),
        )),
        scale_sets: Arc::new(Factory(remote)),
    });
    harness.use_registrations(registrations);
    harness.runtime.installations = Probe::present();
    event
}
