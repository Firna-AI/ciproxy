//! Boat's account and spend rules used when preparing a new sandbox.

use rust_decimal::Decimal;

use super::OverflowProvider;

#[test]
fn boat_trial_shortens_the_lifetime_to_two_hours() {
    let rule = OverflowProvider::Boat.descriptor().lifetime;
    assert_eq!(rule.seconds(true), 7200);
    assert_eq!(rule.seconds(false), 21600);
}

#[test]
fn boat_requires_a_published_start_fee_even_for_a_free_start() {
    let rule = OverflowProvider::Boat.descriptor().start_fee;
    assert_eq!(rule.admission_bound(None), None);
    assert_eq!(
        rule.admission_bound(Some(Decimal::ZERO)),
        Some(Decimal::ZERO)
    );
}
