//! Pure measured billing coverage and provider lifetime bounds.

use chrono::Duration;
use ciproxy_store_interface::overflow::billing::MeasuredBillingInterval;
use ciproxy_store_interface::overflow::record::StartRecord;
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// Derive the continuous billed prefix, clamped to the last observation.
pub(crate) fn billing_prefix(record: &StartRecord) -> (Option<MeasuredBillingInterval>, bool) {
    let Some(last) = record.runtime.iter().max_by_key(|charge| charge.until) else {
        return (
            record.runtime_complete.then(|| {
                MeasuredBillingInterval::new(record.starts_at, record.starts_at, record.starts_at).0
            }),
            false,
        );
    };
    let latest = last.until;
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
            let (interval, clamped) = MeasuredBillingInterval::new(record.starts_at, end, latest);
            (Some(interval), clamped)
        }
        None => {
            let (interval, _) = MeasuredBillingInterval::new(record.starts_at, latest, latest);
            (Some(interval), true)
        }
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
