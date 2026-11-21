//! UTC billing boundaries and calendar-month Free anchors.

use chrono::{DateTime, Datelike, Months, Utc};
use ciproxy_store_interface::metering::history::Period;

/// Find the Free month containing `at`, preserving the original monthly anchor.
///
/// Return None before Free starts or outside the calendar representation.
pub fn free_period(anchor: DateTime<Utc>, at: DateTime<Utc>) -> Option<Period> {
    if at < anchor {
        return None;
    }
    let months = (at.year() - anchor.year()) * 12 + at.month() as i32 - anchor.month() as i32;
    let mut offset = u32::try_from(months).ok()?;
    let mut starts_at = anchor.checked_add_months(Months::new(offset))?;
    if starts_at > at {
        offset = offset.checked_sub(1)?;
        starts_at = anchor.checked_add_months(Months::new(offset))?;
    }
    Some(Period {
        starts_at,
        ends_at: anchor.checked_add_months(Months::new(offset.checked_add(1)?))?,
    })
}

#[cfg(test)]
#[path = "_tests_/period_tests.rs"]
mod period_tests;
