//! Calendar-month costs from measured runtime and the exact pinned hourly price.

use chrono::{DateTime, Datelike, Months, Utc};
use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartPhase, StartRecord};
use rust_decimal::Decimal;

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
    let month = month_start(now)?.date_naive();
    let mut spend = Spend {
        known_usd: Decimal::ZERO,
        unknown_start_charges: Vec::new(),
        unmeasured_runtime: Vec::new(),
    };
    for start in starts
        .iter()
        .filter(|start| start.phase != StartPhase::Refused)
    {
        if month_start(start.starts_at)?.date_naive() == month && start.sandbox_id.is_some() {
            match start.start_usd {
                Some(fee) => spend.known_usd += fee,
                None => spend.unknown_start_charges.push(start.delivery_id),
            }
        }
        for runtime in start
            .runtime
            .iter()
            .filter(|runtime| runtime.month == month)
        {
            spend.known_usd += runtime.usd;
        }
        let relevant = start
            .ends_at
            .is_none_or(|end| end >= month_start(now).unwrap_or(now));
        if relevant && !start.runtime_complete {
            spend.unmeasured_runtime.push(start.delivery_id);
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
