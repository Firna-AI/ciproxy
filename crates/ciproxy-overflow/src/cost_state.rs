//! Pure measured billing coverage and provider lifetime bounds.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::overflow::record::StartRecord;
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// Derive the continuous billed prefix, clamped to the last observation.
pub(crate) fn billing_prefix(record: &StartRecord) -> (Option<DateTime<Utc>>, bool) {
    let Some(last) = record.runtime.iter().max_by_key(|charge| charge.until) else {
        return (record.runtime_complete.then_some(record.starts_at), false);
    };
    let latest = last.until.max(record.starts_at);
    let usd = record
        .runtime
        .iter()
        .try_fold(Decimal::ZERO, |total, charge| total.checked_add(charge.usd));
    let end = usd.and_then(|usd| {
        if usd.is_zero() {
            return Some(record.starts_at);
        }
        usd.checked_mul(Decimal::from(3_600_000_000_000i64))?
            .checked_div(record.size.hourly_usd)?
            .to_i64()
            .and_then(|nanos| {
                record
                    .starts_at
                    .checked_add_signed(Duration::nanoseconds(nanos))
            })
    });
    match end {
        Some(end) => {
            let bounded = end.max(record.starts_at).min(latest);
            (Some(bounded), bounded != end)
        }
        None => (Some(latest), true),
    }
}

/// Missing data preserves completeness only after a final stopped reading.
pub(crate) fn missing_observation(record: &mut StartRecord) {
    record.runtime_complete = record
        .runtime
        .iter()
        .max_by_key(|charge| charge.until)
        .is_some_and(|charge| !charge.running);
}

/// Keep the earliest proven upper bound for one known sandbox's lifecycle.
pub(crate) fn observe_existence(record: &mut StartRecord, at: DateTime<Utc>) -> bool {
    if record.sandbox_id.is_none() {
        return false;
    }
    let Some(bound) = at.checked_add_signed(Duration::seconds(i64::from(record.lifetime_seconds)))
    else {
        return false;
    };
    if record
        .lifetime_ends_by
        .is_none_or(|previous| bound < previous)
    {
        record.lifetime_ends_by = Some(bound);
        return true;
    }
    false
}
