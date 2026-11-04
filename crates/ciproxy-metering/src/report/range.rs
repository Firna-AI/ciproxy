//! The three UTC report ranges and their default selection.

use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, Utc};

/// A Savings page range, with the last thirty days selected initially.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReportRange {
    /// Thirty UTC dates, including today's date.
    #[default]
    Last30Days,
    /// The current calendar month through today's date.
    ThisMonth,
    /// The complete previous calendar month.
    LastMonth,
}

/// Half-open UTC date boundaries for daily accounting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateRange {
    /// The first included UTC date.
    pub from: NaiveDate,
    /// The first excluded UTC date.
    pub until: NaiveDate,
}

/// Resolve report dates from an explicit time without reading a clock.
pub fn report_range(selection: ReportRange, at: DateTime<Utc>) -> Option<DateRange> {
    let today = at.date_naive();
    let tomorrow = today.succ_opt()?;
    let month = today.with_day(1)?;
    Some(match selection {
        ReportRange::Last30Days => DateRange {
            from: today.checked_sub_signed(Duration::days(29))?,
            until: tomorrow,
        },
        ReportRange::ThisMonth => DateRange {
            from: month,
            until: tomorrow,
        },
        ReportRange::LastMonth => DateRange {
            from: month.checked_sub_months(Months::new(1))?,
            until: month,
        },
    })
}

/// The first of the ninety retained UTC dates, including the current date.
pub fn retention_start(at: DateTime<Utc>) -> Option<NaiveDate> {
    at.date_naive().checked_sub_signed(Duration::days(89))
}

pub(crate) fn midnight(day: NaiveDate) -> DateTime<Utc> {
    day.and_time(chrono::NaiveTime::MIN).and_utc()
}

#[cfg(test)]
#[path = "../_tests_/range_tests.rs"]
mod range_tests;
