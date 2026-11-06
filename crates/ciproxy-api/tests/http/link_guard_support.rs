//! Real-store execution boundaries reject old resources of an inactive organization.

use std::sync::Arc;

use ciproxy_github::error::Error;
use ciproxy_github::listener::{DurableListener, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::registrations::{RegistrationService, RunnerRegistrations};
use ciproxy_store_interface::scale_sets::ScaleSet;
use tokio::sync::watch;
use uuid::Uuid;

use super::fixture::Fixture;
use super::label_retirement_support::Queue;

pub(super) async fn refuses_runners(fixture: &Fixture, organization: Uuid) {
    let label = fixture
        .database
        .store
        .labels(fixture.workspace)
        .unwrap()
        .remove(0);
    let set = ScaleSet {
        id: Uuid::new_v4(),
        organization_id: organization,
        label_id: label.id,
        github_id: 60001,
        runner_group_id: 47,
    };
    fixture
        .database
        .store
        .create_scale_set(fixture.workspace, &set)
        .unwrap();
    let target = ListenerTarget {
        workspace_id: fixture.workspace,
        organization_id: organization,
        scale_set_id: set.id,
    };
    let store = Arc::new(StoreAdapter::new(
        fixture.database.store.clone(),
        fixture.runtime.clone(),
    ));
    let registrations = RegistrationService {
        store: store.clone(),
        scale_sets: fixture.state.scale_sets.clone(),
    };
    assert!(matches!(
        registrations.create(target, "must-not-start").await,
        Err(Error::OrganizationInactive)
    ));
    let listener = DurableListener {
        store,
        rest: fixture.remote.clone(),
        scale_sets: fixture.state.scale_sets.clone(),
        output: Arc::new(Queue::default()),
    };
    let (_stop, signal) = watch::channel(false);
    assert!(matches!(
        listener
            .run(
                target,
                ListenerOptions {
                    owner: "held".to_owned(),
                    capacity: 2,
                    max_polls: Some(1)
                },
                signal
            )
            .await,
        Err(Error::OrganizationInactive)
    ));
    assert!(
        fixture
            .database
            .store
            .listener_scale_sets(fixture.workspace, organization)
            .unwrap()
            .is_empty()
    );
}
