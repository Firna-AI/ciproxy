//! Saved routing choices and limits across the label order migration.

use ciproxy_schema::migrations::MIGRATIONS;
use ciproxy_store_interface::labels::{LabelPlace, WhenNothingFits};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_provider_settings::OverflowProviderSettings;
use ciproxy_store_interface::secrets::EncryptedOverflowKey;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use diesel::migration::MigrationVersion;
use diesel::prelude::*;
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use super::support::{TestDatabase, account};

#[test]
fn migration_preserves_switch_pairs_limits_and_the_existing_boat_offer() {
    let database = TestDatabase::new();
    let mut connection = PgConnection::establish(&database.url).unwrap();
    let label_order = MigrationVersion::from("202611231601");
    for migration in connection
        .pending_migrations(MIGRATIONS)
        .unwrap()
        .into_iter()
        .take_while(|migration| migration.name().version() < label_order)
    {
        connection.run_migration(migration.as_ref()).unwrap();
    }
    connection
        .batch_execute(include_str!("label_order_migration.sql"))
        .unwrap();
    let store = PgStoreFactory.connect(&database.url).unwrap();
    let workspace = Uuid::from_u128(1);
    let labels = store.labels(workspace).unwrap();
    let boat = LabelPlace::Provider(OverflowProvider::Boat);
    let machine = LabelPlace::Machine;
    let actual: Vec<_> = labels
        .iter()
        .map(|label| {
            (
                label.name.as_str(),
                label.order.as_slice(),
                label.when_nothing_fits,
            )
        })
        .collect();
    let wait = WhenNothingFits::Wait;
    assert_eq!(
        actual,
        [
            ("both", &[machine, boat][..], wait),
            ("empty", &[], wait),
            ("machine", &[machine], wait),
            ("macos", &[machine], wait),
            ("only-boat", &[boat], WhenNothingFits::LargestOverflow),
        ]
    );
    for (id, limit) in [(1, Some(12_345_678)), (2, None), (3, None)] {
        assert_eq!(
            store
                .overflow_provider_settings(Uuid::from_u128(id), OverflowProvider::Boat)
                .unwrap(),
            Some(OverflowProviderSettings {
                spend_limit_usd_micros: limit,
                offered: true
            }),
            "workspace {id}"
        );
    }
    store
        .connect_overflow_provider(
            workspace,
            &account(),
            &EncryptedOverflowKey::new(vec![1]),
            50_000_000,
        )
        .unwrap();
    assert_eq!(store.labels(workspace).unwrap(), labels);
}
