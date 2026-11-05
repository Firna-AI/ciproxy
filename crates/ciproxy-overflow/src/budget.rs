//! Estimate-only lifetime windows, separate from measured and lifecycle proof.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::overflow::record::StartRecord;
use rust_decimal::Decimal;

use crate::error::{Error, Result};
use crate::spend::{month_start, next_month};

/// Budget expiry uses dispatch until a trusted usage read supplies creation time.
pub(crate) fn expires_at(start: &StartRecord) -> Result<DateTime<Utc>> {
    let parameters = start.dispatch.as_ref().ok_or(Error::OwnershipUnknown)?;
    let lifetime = Duration::seconds(i64::from(parameters.lifetime_seconds));
    let original = start
        .starts_at
        .checked_add_signed(lifetime)
        .ok_or(Error::InvalidProviderData)?;
    let later_running = start
        .runtime
        .iter()
        .filter(|charge| charge.running)
        .map(|charge| charge.until)
        .chain(start.running_observed_at)
        .filter(|at| *at > original)
        .max();
    later_running.map_or(Ok(original), |at| {
        at.checked_add_signed(lifetime)
            .ok_or(Error::InvalidProviderData)
    })
}

/// A full-lifetime runtime bound is split at UTC calendar-month boundaries.
pub(crate) fn runtime_bound(start: &StartRecord, month: DateTime<Utc>) -> Result<Decimal> {
    let parameters = start.dispatch.as_ref().ok_or(Error::OwnershipUnknown)?;
    let until = expires_at(start)?.min(next_month(month)?);
    let since = start.starts_at.max(month_start(month)?);
    let nanos = (until - since)
        .num_nanoseconds()
        .ok_or(Error::InvalidProviderData)?
        .max(0);
    let lifetime = i64::from(parameters.lifetime_seconds) * 1_000_000_000;
    Ok(
        parameters.size.hourly_usd * Decimal::from(nanos.min(lifetime))
            / Decimal::from(3_600_000_000_000i64),
    )
}

/// Complete observed or normally computed runtime needs no remaining runtime bound.
pub(crate) fn runtime_final(start: &StartRecord) -> bool {
    let running_at = start
        .running_observed_at
        .into_iter()
        .chain(
            start
                .runtime
                .iter()
                .filter(|charge| charge.running)
                .map(|charge| charge.until),
        )
        .max();
    start
        .runtime_total
        .as_ref()
        .is_some_and(|total| !total.running && running_at.is_none_or(|at| at <= total.until))
        || (start.runtime_complete
            && (start
                .ends_at
                .is_some_and(|end| running_at.is_none_or(|at| at <= end))
                || start
                    .runtime
                    .iter()
                    .max_by_key(|charge| charge.until)
                    .is_some_and(|charge| {
                        !charge.running && running_at.is_none_or(|at| at <= charge.until)
                    })))
}

/// Keep exact single-month totals and conservative unallocated cross-month costs apart.
pub(crate) fn runtime_evidence(
    start: &StartRecord,
    month: DateTime<Utc>,
) -> Result<(Decimal, Decimal)> {
    let next = next_month(month)?;
    let measured = start
        .runtime
        .iter()
        .filter(|charge| charge.month == month.date_naive())
        .map(|charge| charge.usd)
        .sum::<Decimal>();
    let Some(total) = &start.runtime_total else {
        return Ok((measured, Decimal::ZERO));
    };
    if total.since >= month && total.until <= next {
        return Ok((measured.max(total.usd), Decimal::ZERO));
    }
    let covered = start
        .runtime
        .iter()
        .any(|charge| charge.month == month.date_naive() && charge.until >= total.until.min(next));
    if total.since >= next || total.until <= month || covered {
        return Ok((measured, Decimal::ZERO));
    }
    let elsewhere = start
        .runtime
        .iter()
        .filter(|charge| charge.month != month.date_naive() && charge.until <= total.until)
        .map(|charge| charge.usd)
        .sum::<Decimal>();
    Ok((measured, (total.usd - elsewhere).max(Decimal::ZERO)))
}
