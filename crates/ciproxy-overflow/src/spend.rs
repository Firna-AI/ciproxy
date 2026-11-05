//! Calendar-month costs from measured runtime and the exact pinned hourly price.

use chrono::{DateTime, Datelike, Months, Utc};
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase, StartRecord};
use rust_decimal::Decimal;

use crate::budget;
use crate::error::{Error, Result};
use crate::view::Spend;

pub(crate) fn month_start(at: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let day = at
        .date_naive()
        .with_day(1)
        .ok_or(Error::InvalidProviderData)?;
    Ok(DateTime::from_naive_utc_and_offset(
        day.and_hms_opt(0, 0, 0).ok_or(Error::InvalidProviderData)?,
        Utc,
    ))
}

pub(crate) fn next_month(at: DateTime<Utc>) -> Result<DateTime<Utc>> {
    month_start(at)?
        .checked_add_months(Months::new(1))
        .ok_or(Error::InvalidProviderData)
}

pub(crate) fn current(starts: &[StartRecord], now: DateTime<Utc>) -> Result<Spend> {
    let month_at = month_start(now)?;
    let month = month_at.date_naive();
    let mut spend = Spend {
        confirmed_usd: Decimal::ZERO,
        pending_usd: Decimal::ZERO,
        settled_bound_usd: Decimal::ZERO,
        unbounded_starts: Vec::new(),
        unknown_start_charges: Vec::new(),
        unmeasured_runtime: Vec::new(),
    };
    for start in starts.iter().filter(|start| {
        !matches!(
            start.phase,
            StartPhase::Refused | StartPhase::CancelledUnsent
        )
    }) {
        let mut unsent;
        let start = if start.phase == StartPhase::Unsent {
            unsent = start.clone();
            unsent.starts_at = now;
            &unsent
        } else {
            start
        };
        let fee_in_month = month_start(start.starts_at)?.date_naive() == month;
        let (runtime, unallocated) = budget::runtime_evidence(start, month_at)?;
        spend.confirmed_usd += runtime;
        if fee_in_month {
            spend.confirmed_usd += start.start_usd.unwrap_or(Decimal::ZERO);
        }
        let runtime_bound = if budget::runtime_final(start) {
            Decimal::ZERO
        } else {
            budget::runtime_bound(start, month_at)?
        }
        .max(unallocated);
        let relevant = fee_in_month || runtime_bound > Decimal::ZERO || runtime > Decimal::ZERO;
        if !relevant {
            continue;
        }
        if !start.runtime_complete {
            spend.unmeasured_runtime.push(start.delivery_id);
        }
        let fee = if fee_in_month && start.start_usd.is_none() {
            spend.unknown_start_charges.push(start.delivery_id);
            match start.counted_start_fee_usd() {
                Some(bound) => bound,
                None => {
                    spend.unbounded_starts.push(start.delivery_id);
                    Decimal::ZERO
                }
            }
        } else {
            Decimal::ZERO
        };
        let remaining_runtime = (runtime_bound - runtime).max(Decimal::ZERO);
        let remaining = fee + remaining_runtime;
        if budget::expires_at(start)? <= now {
            spend.settled_bound_usd += remaining;
        } else {
            spend.pending_usd += remaining;
        }
    }
    Ok(spend)
}

/// Price only the known running interval after the last provider observation.
pub(crate) fn extend(record: &mut StartRecord, until: DateTime<Utc>) -> Result<()> {
    let Some(last) = record
        .runtime
        .iter()
        .max_by_key(|charge| charge.until)
        .cloned()
    else {
        record.runtime_complete = false;
        return Ok(());
    };
    if !last.running {
        record.runtime_complete = true;
        return Ok(());
    }
    let mut from = last.until;
    while from < until {
        let end = next_month(from)?.min(until);
        let nanos = (end - from)
            .num_nanoseconds()
            .ok_or(Error::InvalidProviderData)?;
        let usd =
            record.size.hourly_usd * Decimal::from(nanos) / Decimal::from(3_600_000_000_000i64);
        let month = month_start(from)?.date_naive();
        if let Some(charge) = record
            .runtime
            .iter_mut()
            .find(|charge| charge.month == month)
        {
            charge.usd += usd;
            charge.until = end;
            charge.running = false;
        } else {
            record.runtime.push(RuntimeCharge {
                month,
                usd,
                until: end,
                running: false,
            });
        }
        from = end;
    }
    record.runtime_complete = true;
    Ok(())
}
