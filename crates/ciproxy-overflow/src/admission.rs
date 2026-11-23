//! Cost of the next start, with the same UTC month split as dispatch.

use chrono::{DateTime, Duration, Utc};
use ciproxy_scaler::provider::StartBound;
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::admission::ReservationBudget;
use ciproxy_store_interface::overflow::record::StartRecord;
use rust_decimal::Decimal;

use crate::budget::runtime_in_month;
use crate::catalog::micros;
use crate::error::{Error, Result};
use crate::spend;

/// Bound one catalog start in microdollars for each month it can run in.
pub fn start_bound(
    now: DateTime<Utc>,
    size: &SizeOffer,
    lifetime_seconds: u32,
    start_fee_usd: Option<Decimal>,
) -> Result<StartBound> {
    let bound = monthly_bound(now, size, lifetime_seconds, start_fee_usd)?;
    Ok(StartBound {
        current_month: micros(bound.current_month_usd)?,
        next_month: bound.next_month_usd.map(micros).transpose()?,
    })
}

pub(crate) fn catalog_bound(
    state: &AccountState,
    size: &SizeOffer,
    now: DateTime<Utc>,
) -> Result<StartBound> {
    let descriptor = state.provider.descriptor();
    start_bound(
        now,
        size,
        descriptor.lifetime.seconds(state.trial),
        descriptor
            .start_fee
            .admission_bound(state.start_fee_bound_usd),
    )
}

pub(crate) fn required_budget(
    starts: &[StartRecord],
    record: &StartRecord,
) -> Result<ReservationBudget> {
    let parameters = record.dispatch.as_ref().ok_or(Error::OwnershipUnknown)?;
    let bound = monthly_bound(
        record.starts_at,
        &parameters.size,
        parameters.lifetime_seconds,
        record.counted_start_fee_usd(),
    )?;
    let starts = starts
        .iter()
        .filter(|start| start.delivery_id != record.delivery_id);
    let current =
        spend::current(record.provider, starts.clone(), record.starts_at)?.counted_usd()?;
    let next = bound
        .next_month_usd
        .map(|bound| -> Result<_> {
            Ok(bound
                + spend::current(
                    record.provider,
                    starts,
                    spend::next_month(record.starts_at)?,
                )?
                .counted_usd()?)
        })
        .transpose()?;
    Ok(ReservationBudget {
        current_month_usd: current + bound.current_month_usd,
        next_month_usd: next,
    })
}

fn monthly_bound(
    starts_at: DateTime<Utc>,
    size: &SizeOffer,
    lifetime_seconds: u32,
    start_fee_usd: Option<Decimal>,
) -> Result<ReservationBudget> {
    let fee = start_fee_usd.ok_or(Error::AccountingIncomplete)?;
    let expires_at = starts_at
        .checked_add_signed(Duration::seconds(i64::from(lifetime_seconds)))
        .ok_or(Error::InvalidProviderData)?;
    let runtime = |month| {
        runtime_in_month(
            size.hourly_usd,
            starts_at,
            expires_at,
            lifetime_seconds,
            month,
        )
    };
    let next = spend::next_month(starts_at)?;
    Ok(ReservationBudget {
        current_month_usd: fee + runtime(starts_at)?,
        next_month_usd: (next < expires_at).then(|| runtime(next)).transpose()?,
    })
}
