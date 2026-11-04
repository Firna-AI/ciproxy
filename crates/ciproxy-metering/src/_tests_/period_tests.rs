//! Free anchor recovery and confirmed billing transitions.

use ciproxy_store_interface::metering::history::Period;

use crate::period::{PeriodEvent, change_period, free_period};
use crate::test_support::at;

#[test]
fn free_period_recovers_original_day_after_short_months() {
    let anchor = at("2027-01-31T12:00:00Z");
    for (now, start, end) in [
        (
            "2027-02-27T23:00:00Z",
            "2027-01-31T12:00:00Z",
            "2027-02-28T12:00:00Z",
        ),
        (
            "2027-02-28T12:00:00Z",
            "2027-02-28T12:00:00Z",
            "2027-03-31T12:00:00Z",
        ),
        (
            "2027-04-01T00:00:00Z",
            "2027-03-31T12:00:00Z",
            "2027-04-30T12:00:00Z",
        ),
    ] {
        assert_eq!(
            free_period(anchor, at(now)),
            Some(Period {
                starts_at: at(start),
                ends_at: at(end)
            }),
            "{now}"
        );
    }
    assert_eq!(free_period(anchor, at("2027-01-31T11:59:59Z")), None);
}

#[test]
fn confirmed_upgrade_resets_free_period_and_paid_change_keeps_it() {
    let free = Period {
        starts_at: at("2026-11-01T00:00:00Z"),
        ends_at: at("2026-12-01T00:00:00Z"),
    };
    let stripe = Period {
        starts_at: at("2026-11-04T15:00:00Z"),
        ends_at: at("2026-12-04T15:00:00Z"),
    };
    let upgrade = change_period(free, PeriodEvent::PaidConfirmed(stripe)).unwrap();
    assert_eq!(upgrade.closed.unwrap().ends_at, at("2026-11-04T15:00:00Z"));
    assert_eq!(upgrade.current, stripe);
    let paid = change_period(stripe, PeriodEvent::PaidTierChanged).unwrap();
    assert_eq!(paid.closed, None);
    assert_eq!(paid.current, stripe);
    let ended = change_period(
        stripe,
        PeriodEvent::SubscriptionEnded(at("2026-12-04T15:00:00Z")),
    )
    .unwrap();
    assert_eq!(
        ended.current,
        Period {
            starts_at: at("2026-12-04T15:00:00Z"),
            ends_at: at("2027-01-04T15:00:00Z")
        }
    );
}
