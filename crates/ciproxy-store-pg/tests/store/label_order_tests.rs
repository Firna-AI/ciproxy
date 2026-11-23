//! Store writes and defaults for ordered label places.

use ciproxy_protocol::platform::Platform;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::labels::{Label, LabelPlace, WhenNothingFits};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::vm_size::VmSize;
use uuid::Uuid;

use super::support::Database;

const BOAT: LabelPlace = LabelPlace::Provider(OverflowProvider::Boat);

fn label(name: &str, platform: Platform, order: Vec<LabelPlace>) -> Label {
    Label {
        id: Uuid::new_v4(),
        name: name.to_owned(),
        platform,
        size: VmSize {
            vcpus: 2,
            memory_gb: 8,
            disk_gb: 20,
        },
        order,
        when_nothing_fits: WhenNothingFits::Wait,
    }
}

#[test]
fn replaces_label_order_and_rolls_back_duplicate_places() {
    let database = Database::new();
    let workspace = database.workspace().id;
    let store = &database.store;
    let machine = LabelPlace::Machine;
    let mut saved = label("linux", Platform::LinuxX64, vec![machine, BOAT]);
    store.create_label(workspace, &saved).unwrap();

    saved.order = vec![BOAT, machine];
    store.update_label(workspace, &saved).unwrap();
    assert_eq!(store.labels(workspace).unwrap(), [saved.clone()]);

    let mut duplicate = saved.clone();
    duplicate.order = vec![machine, machine];
    duplicate.size.vcpus = 4;
    assert!(matches!(
        store.update_label(workspace, &duplicate),
        Err(Error::Conflict)
    ));
    assert_eq!(store.labels(workspace).unwrap(), [saved.clone()]);

    saved.order.clear();
    store.update_label(workspace, &saved).unwrap();
    assert_eq!(store.labels(workspace).unwrap(), [saved]);
}

#[test]
fn first_offer_appends_to_linux_orders_and_repeated_offers_keep_edits() {
    let database = Database::new();
    let workspace = database.workspace().id;
    let other = database.workspace().id;
    let store = &database.store;
    let machine = LabelPlace::Machine;
    let mut empty = label("empty", Platform::LinuxX64, vec![]);
    let mut linux = label("linux", Platform::LinuxX64, vec![machine]);
    let macos = label("macos", Platform::MacosArm64, vec![machine]);
    let with_boat = label("with-boat", Platform::LinuxX64, vec![BOAT, machine]);
    for label in [&empty, &linux, &macos, &with_boat] {
        store.create_label(workspace, label).unwrap();
    }
    let foreign = label("foreign", Platform::LinuxX64, vec![machine]);
    store.create_label(other, &foreign).unwrap();

    store
        .offer_overflow_provider(workspace, OverflowProvider::Boat)
        .unwrap();
    linux.order.push(BOAT);
    empty.order.push(BOAT);
    assert_eq!(
        store.labels(workspace).unwrap(),
        [
            empty.clone(),
            linux.clone(),
            macos.clone(),
            with_boat.clone()
        ]
    );
    assert_eq!(store.labels(other).unwrap(), [foreign]);

    linux.order = vec![machine];
    store.update_label(workspace, &linux).unwrap();
    store
        .offer_overflow_provider(workspace, OverflowProvider::Boat)
        .unwrap();
    assert_eq!(
        store.labels(workspace).unwrap(),
        [empty, linux, macos, with_boat]
    );
}

#[test]
fn customer_defaults_use_only_offered_providers_and_only_on_linux() {
    let database = Database::new();
    let workspace = database.workspace().id;
    let store = &database.store;
    let machine = LabelPlace::Machine;
    store
        .save_overflow_provider_spend_limit(workspace, OverflowProvider::Boat, Some(25_000_000))
        .unwrap();
    let mut before = label("before", Platform::LinuxX64, vec![]);
    store
        .create_label_with_default_order(workspace, &before)
        .unwrap();
    before.order = vec![machine];
    assert_eq!(store.labels(workspace).unwrap(), [before.clone()]);

    store
        .offer_overflow_provider(workspace, OverflowProvider::Boat)
        .unwrap();
    let mut linux = label("linux", Platform::LinuxX64, vec![]);
    let mut macos = label("macos", Platform::MacosArm64, vec![]);
    store
        .create_label_with_default_order(workspace, &linux)
        .unwrap();
    store
        .create_label_with_default_order(workspace, &macos)
        .unwrap();
    before.order = vec![machine, BOAT];
    linux.order = vec![machine, BOAT];
    macos.order = vec![machine];
    assert_eq!(store.labels(workspace).unwrap(), [before, linux, macos]);
}
