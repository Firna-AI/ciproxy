//! Workspace and provider ownership of saved overflow policy.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_provider_settings::OverflowProviderSettings;

use super::support::Database;

#[test]
fn saves_one_provider_limit_per_workspace_and_preserves_the_offer() {
    let database = Database::new();
    let first = database.workspace().id;
    let second = database.workspace().id;
    let store = &database.store;
    let provider = OverflowProvider::Boat;
    store
        .save_overflow_provider_spend_limit(first, provider, Some(20_000_000))
        .unwrap();
    store
        .save_overflow_provider_spend_limit(second, provider, Some(30_000_000))
        .unwrap();
    assert_eq!(
        store.overflow_provider_settings(first, provider).unwrap(),
        Some(OverflowProviderSettings {
            spend_limit_usd_micros: Some(20_000_000),
            offered: false,
        })
    );

    store.offer_overflow_provider(first, provider).unwrap();
    store
        .save_overflow_provider_spend_limit(first, provider, Some(40_000_000))
        .unwrap();
    assert!(matches!(
        store.save_overflow_provider_spend_limit(first, provider, Some(0)),
        Err(Error::InvalidValue)
    ));
    assert_eq!(
        store.overflow_provider_settings(first, provider).unwrap(),
        Some(OverflowProviderSettings {
            spend_limit_usd_micros: Some(40_000_000),
            offered: true,
        })
    );
    assert_eq!(
        store.overflow_provider_settings(second, provider).unwrap(),
        Some(OverflowProviderSettings {
            spend_limit_usd_micros: Some(30_000_000),
            offered: false,
        })
    );
}
