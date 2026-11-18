//! Free anchor recovery across short calendar months.

use ciproxy_store_interface::metering::history::Period;

use crate::period::free_period;
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
