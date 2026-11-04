//! Hand-worked paid tier and add-on price comparisons.

use crate::pricing::{Tier, cheapest_offer};
use crate::test_support::usd;

#[test]
fn cheapest_paid_offer_covers_starts_and_machines() {
    for (starts, machines, tier, blocks, extras, amount) in [
        ("0", 0, Tier::Team, 0, 0, "49"),
        ("51840", 1, Tier::Team, 3, 0, "79"),
        ("150000", 10, Tier::Business, 0, 0, "299"),
        ("200001", 12, Tier::Business, 6, 2, "399"),
    ] {
        let offer = cheapest_offer(usd(starts), machines, Tier::Free);
        assert_eq!(
            (
                offer.plan.tier,
                offer.plan.add_ons.start_blocks,
                offer.plan.add_ons.machines,
                offer.monthly_usd
            ),
            (tier, blocks, extras, usd(amount)),
            "{starts}/{machines}"
        );
        assert!(offer.starts >= usd(starts));
        assert!(offer.machines >= u128::from(machines));
    }
}

#[test]
fn equal_price_offer_preserves_current_paid_tier() {
    let current = cheapest_offer(usd("130000"), 10, Tier::Business);
    assert_eq!(
        (current.plan.tier, current.monthly_usd),
        (Tier::Business, usd("299"))
    );
    let other = cheapest_offer(usd("130000"), 10, Tier::Team);
    assert_eq!(
        (
            other.plan.tier,
            other.plan.add_ons.start_blocks,
            other.plan.add_ons.machines,
            other.monthly_usd
        ),
        (Tier::Team, 11, 7, usd("299"))
    );
}
