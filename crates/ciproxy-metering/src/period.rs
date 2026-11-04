//! UTC billing boundaries and calendar-month Free anchors.

use chrono::{DateTime, Datelike, Months, Utc};
use ciproxy_store_interface::metering::history::Period;

/// A billing event that can change a workspace's current period.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeriodEvent {
    /// Stripe confirmed the first paid period after Free.
    PaidConfirmed(Period),
    /// A paid-tier change preserves the existing period.
    PaidTierChanged,
    /// Stripe ended the subscription and Free starts at this instant.
    SubscriptionEnded(DateTime<Utc>),
}

/// The old closed period, when applicable, and the current period.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeriodChange {
    /// The prior period with its final exclusive end.
    pub closed: Option<Period>,
    /// The period that counts subsequent starts.
    pub current: Period,
}

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

/// Apply confirmed billing boundaries without moving starts between periods.
pub fn change_period(current: Period, event: PeriodEvent) -> Option<PeriodChange> {
    let next = match event {
        PeriodEvent::PaidTierChanged => {
            return Some(PeriodChange {
                closed: None,
                current,
            });
        }
        PeriodEvent::PaidConfirmed(period) => period,
        PeriodEvent::SubscriptionEnded(at) => free_period(at, at)?,
    };
    Some(PeriodChange {
        closed: Some(Period {
            starts_at: current.starts_at,
            ends_at: next.starts_at,
        }),
        current: next,
    })
}

#[cfg(test)]
#[path = "_tests_/period_tests.rs"]
mod period_tests;
