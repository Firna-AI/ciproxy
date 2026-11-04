//! Boat key custody, mandatory spending limits, and sandbox-aware rotation.

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use ciproxy_store_interface::secrets::EncryptedBoatKey;

use super::support::{Database, account, at, runner};

#[test]
fn boat_rotation_retains_live_keys_and_requires_a_spend_limit() {
    let database = Database::new();
    let store = &database.store;
    let owner = database.organization();
    let (label, scale_set) = database.label_and_scale_set(&owner);
    let old = account();
    let old_key = EncryptedBoatKey::new(vec![10, 20, 30]);
    assert!(matches!(
        store.connect_boat(owner.id, &old, &old_key, 0),
        Err(Error::InvalidValue)
    ));
    assert_eq!(store.boat_accounts(owner.id).unwrap(), vec![]);
    assert_eq!(
        store
            .settings(owner.id)
            .unwrap()
            .boat_spend_limit_usd_micros,
        None
    );
    store
        .connect_boat(owner.id, &old, &old_key, 200_000_000)
        .unwrap();
    let live = runner(
        &label,
        &scale_set,
        RunnerPlace::Boat {
            key_id: Some(old.id),
            sandbox_id: "sandbox-live".to_owned(),
            size_name: "default".to_owned(),
        },
    );
    store.create_runner(owner.id, &live).unwrap();
    let mut new = account();
    new.connected_at += Duration::minutes(1);
    let new_key = EncryptedBoatKey::new(vec![40, 50, 60]);
    store
        .connect_boat(owner.id, &new, &new_key, 300_000_000)
        .unwrap();
    let mut expected_old = old.clone();
    expected_old.retired_at = Some(new.connected_at);
    assert_eq!(
        store.boat_accounts(owner.id).unwrap(),
        vec![expected_old, new.clone()]
    );
    assert_eq!(
        store.encrypted_boat_key(owner.id, old.id).unwrap(),
        Some(old_key.clone())
    );
    assert_eq!(
        store.encrypted_boat_key(owner.id, new.id).unwrap(),
        Some(new_key)
    );
    assert_eq!(store.delete_unused_boat_keys(owner.id).unwrap(), 0);
    store.reject_boat_key(owner.id, new.id, at()).unwrap();
    new.rejected_at = Some(at());
    assert_eq!(store.boat_accounts(owner.id).unwrap()[1], new);
    store.disconnect_boat(owner.id, at()).unwrap();
    assert_eq!(store.delete_unused_boat_keys(owner.id).unwrap(), 1);
    assert_eq!(
        store.encrypted_boat_key(owner.id, old.id).unwrap(),
        Some(old_key)
    );
    assert_eq!(store.encrypted_boat_key(owner.id, new.id).unwrap(), None);
    assert_eq!(
        store
            .settings(owner.id)
            .unwrap()
            .boat_spend_limit_usd_micros,
        Some(300_000_000)
    );
    let mut no_limit = store.settings(owner.id).unwrap();
    no_limit.boat_spend_limit_usd_micros = None;
    assert!(matches!(
        store.save_settings(owner.id, &no_limit),
        Err(Error::InvalidValue)
    ));
    store
        .update_runner(owner.id, live.id, RunnerState::Stopped, live.github_id)
        .unwrap();
    assert_eq!(store.delete_unused_boat_keys(owner.id).unwrap(), 1);
    assert_eq!(store.encrypted_boat_key(owner.id, old.id).unwrap(), None);
    let mut stopped = live;
    stopped.state = RunnerState::Stopped;
    stopped.place = RunnerPlace::Boat {
        key_id: None,
        sandbox_id: "sandbox-live".to_owned(),
        size_name: "default".to_owned(),
    };
    assert_eq!(store.runners(owner.id).unwrap(), vec![stopped]);
}
