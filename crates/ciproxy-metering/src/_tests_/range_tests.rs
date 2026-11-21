//! UTC report and retention date boundaries.

use crate::report::range::{ReportRange, report_range, retention_start};
use crate::test_support::{at, day, range};

#[test]
fn report_ranges_include_today_and_complete_previous_month() {
    let now = at("2027-03-02T00:30:00Z");
    assert_eq!(
        report_range(ReportRange::default(), now),
        Some(range("2027-02-01", "2027-03-03"))
    );
    assert_eq!(
        report_range(ReportRange::ThisMonth, now),
        Some(range("2027-03-01", "2027-03-03"))
    );
    assert_eq!(
        report_range(ReportRange::LastMonth, now),
        Some(range("2027-02-01", "2027-03-01"))
    );
    assert_eq!(retention_start(now), Some(day("2026-12-03")));
}
