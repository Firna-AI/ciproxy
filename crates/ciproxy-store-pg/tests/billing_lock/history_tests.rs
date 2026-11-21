//! Confirmed billing changes preserve an exact monthly plan-price history.

use chrono::Duration;
use ciproxy_store_interface::billing::state::{AddOns, SubscriptionState, Tier};
use rust_decimal::Decimal;

use super::database::Database;
use super::history_support::{intervals, paid, plans};

#[test]
fn billing_commit_tracks_confirmed_prices_and_preserves_equal_totals() {
    let database = Database::new();
    let workspace = database.workspace();
    let mut session = database
        .store
        .try_lock_billing(workspace.id)
        .unwrap()
        .unwrap();
    let initial = session.record().unwrap().state;
    let started = initial.free_started_at;
    let checkout_at = started + Duration::days(1);
    let addon_at = started + Duration::days(4);
    let upgrade_at = started + Duration::days(5);
    let ended_at = started + Duration::days(8);
    assert_eq!(
        intervals(&database, &workspace),
        [(started, None, Decimal::ZERO)]
    );
    session
        .commit(None, Some("evt_pending"), false, false, checkout_at)
        .unwrap();
    let mut confirmed = paid(&initial, checkout_at);
    session
        .commit(
            Some(&confirmed),
            Some("evt_checkout"),
            true,
            false,
            started + Duration::days(3),
        )
        .unwrap();
    assert_eq!(
        intervals(&database, &workspace),
        [
            (started, Some(checkout_at), Decimal::ZERO),
            (checkout_at, None, Decimal::from(89)),
        ]
    );
    let saved = plans(&database, &workspace);
    confirmed.tier = Tier::Business;
    session
        .commit(
            Some(&confirmed),
            Some("evt_checkout"),
            true,
            false,
            addon_at,
        )
        .unwrap();
    assert_eq!(plans(&database, &workspace), saved);
    assert_eq!(session.record().unwrap().state.tier, Tier::Team);
    confirmed.tier = Tier::Team;
    confirmed.status = SubscriptionState::PaymentFailed;
    confirmed.paid_period_end = Some(ended_at + Duration::days(30));
    session
        .commit(Some(&confirmed), None, false, false, addon_at)
        .unwrap();
    assert_eq!(plans(&database, &workspace), saved);
    confirmed.add_ons.start_blocks = 3;
    session
        .commit(Some(&confirmed), None, true, false, addon_at)
        .unwrap();
    confirmed.tier = Tier::Business;
    confirmed.add_ons = AddOns::default();
    session
        .commit(Some(&confirmed), None, true, false, upgrade_at)
        .unwrap();
    let before_equal = plans(&database, &workspace);
    confirmed.tier = Tier::Team;
    confirmed.add_ons.start_blocks = 25;
    session
        .commit(
            Some(&confirmed),
            None,
            true,
            false,
            started + Duration::days(6),
        )
        .unwrap();
    assert_eq!(plans(&database, &workspace), before_equal);
    assert_eq!(session.record().unwrap().state.add_ons.start_blocks, 25);
    confirmed.tier = Tier::Free;
    confirmed.add_ons = AddOns::default();
    confirmed.status = SubscriptionState::Ended;
    confirmed.free_started_at = ended_at;
    confirmed.paid_period_start = None;
    confirmed.paid_period_end = None;
    session
        .commit(
            Some(&confirmed),
            Some("evt_ended"),
            true,
            false,
            started + Duration::days(10),
        )
        .unwrap();
    assert_eq!(
        intervals(&database, &workspace),
        [
            (started, Some(checkout_at), Decimal::ZERO),
            (checkout_at, Some(addon_at), Decimal::from(89)),
            (addon_at, Some(upgrade_at), Decimal::from(99)),
            (upgrade_at, Some(ended_at), Decimal::from(299)),
            (ended_at, None, Decimal::ZERO),
        ]
    );
}

#[test]
fn late_billing_dates_never_put_a_boundary_before_the_open_price() {
    let database = Database::new();
    let workspace = database.workspace();
    let mut session = database
        .store
        .try_lock_billing(workspace.id)
        .unwrap()
        .unwrap();
    let initial = session.record().unwrap().state;
    let started = initial.free_started_at;
    let seed = plans(&database, &workspace).remove(0);
    let mut confirmed = paid(&initial, started - Duration::days(3));
    session
        .commit(
            Some(&confirmed),
            None,
            true,
            false,
            started + Duration::days(1),
        )
        .unwrap();
    let costs = plans(&database, &workspace);
    assert_eq!(costs.len(), 2);
    assert_eq!(
        costs
            .iter()
            .find(|cost| cost.id == seed.id)
            .unwrap()
            .ends_at,
        Some(started)
    );
    let paid_cost = costs.iter().find(|cost| cost.ends_at.is_none()).unwrap();
    assert_eq!(
        (paid_cost.starts_at, paid_cost.monthly_usd),
        (started, Decimal::from(89))
    );
    confirmed.tier = Tier::Free;
    confirmed.add_ons = AddOns::default();
    confirmed.status = SubscriptionState::Ended;
    confirmed.free_started_at = started - Duration::days(4);
    session
        .commit(
            Some(&confirmed),
            None,
            true,
            false,
            started + Duration::days(2),
        )
        .unwrap();
    let costs = plans(&database, &workspace);
    assert_eq!(costs.len(), 3);
    assert_eq!(
        costs
            .iter()
            .find(|cost| cost.id == paid_cost.id)
            .unwrap()
            .ends_at,
        Some(started)
    );
    let free_cost = costs.iter().find(|cost| cost.ends_at.is_none()).unwrap();
    assert_eq!(
        (free_cost.starts_at, free_cost.monthly_usd),
        (started, Decimal::ZERO)
    );
    assert!(
        costs
            .iter()
            .all(|cost| cost.ends_at.is_none_or(|end| end >= cost.starts_at))
    );
}
