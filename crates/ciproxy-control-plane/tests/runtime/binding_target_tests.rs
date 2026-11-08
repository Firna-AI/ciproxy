//! Saved runner targets use the actual registration context across remote changes.

use std::sync::atomic::Ordering;

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::scale_sets::RegistrationTarget;

use super::overflow_support::Harness;
use super::registration_race::Race;

async fn captures_and_replaces(own: bool) {
    let mut harness = Harness::new(own, false).await;
    let mut contact = harness.fixture.contact.clone();
    contact.report.runners.clear();
    contact.report.capacity = Slots(3);
    harness
        .fixture
        .store
        .record_contact(harness.fixture.workspace, harness.fixture.machine, &contact)
        .unwrap();
    let race = Race::install(&mut harness);
    let job = harness.queued();
    harness.runtime.tick().await.unwrap();
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let runner = if own {
        store
            .own_runners(workspace)
            .unwrap()
            .into_iter()
            .find(|intent| intent.runner.github_id == Some(100))
            .unwrap()
            .runner
            .id
    } else {
        store
            .boat_runner_intents(workspace)
            .unwrap()
            .remove(0)
            .runner_id
    };
    assert_eq!(*race.registered.lock().unwrap(), vec![144], "own={own}");
    assert_eq!(
        store.scale_sets(workspace, organization).unwrap()[0].github_id,
        244
    );
    assert_eq!(
        store
            .runner_registration_target(workspace, organization, runner)
            .unwrap(),
        Some(RegistrationTarget {
            organization_id: organization,
            label_id: job.label_id,
            scale_set_id: job.scale_set_id,
            github_id: 144
        }),
        "own={own}: save registration-time context"
    );
    harness.runtime.tick().await.unwrap();
    assert_eq!(
        *race.registered.lock().unwrap(),
        vec![144, 244],
        "own={own}: old remote supply cannot serve current remote demand"
    );
    assert_eq!(
        store
            .runner_registration_target(workspace, organization, runner)
            .unwrap()
            .unwrap()
            .github_id,
        144,
        "retirement and recovery must preserve the original target"
    );
    let starts = if own {
        harness.own_starts.load(Ordering::SeqCst)
    } else {
        harness.ports.creates.lock().unwrap().len()
    };
    assert_eq!(starts, 2);
    assert_eq!(
        harness.ports.deletes.load(Ordering::SeqCst),
        usize::from(!own)
    );
}

#[tokio::test]
async fn own_registration_saves_used_remote_target_and_replaces_idle_supply() {
    captures_and_replaces(true).await;
}

#[tokio::test]
async fn boat_registration_saves_used_remote_target_and_replaces_idle_supply() {
    captures_and_replaces(false).await;
}
