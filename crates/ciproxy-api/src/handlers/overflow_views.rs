//! Credential-free Boat metadata and exact limit conversion.

use chrono::Datelike;
use ciproxy_overflow::error::Error;
use ciproxy_store_interface::settings::OverflowWait;
use rust_decimal::{Decimal, RoundingStrategy};
use uuid::Uuid;

use crate::error;
use crate::state::{ApiState, query};
use crate::wire::money::Cents;
use crate::wire::overflow::{BoatAccountView, BoatSpendView, OverflowSettings};

pub(super) fn limit(cents: Cents) -> error::Result<i64> {
    if cents.0 <= 0 {
        return Err(error::Error::InvalidInput);
    }
    cents
        .0
        .checked_mul(10_000)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or(error::Error::InvalidInput)
}

pub(super) fn result<T>(value: ciproxy_overflow::error::Result<T>) -> error::Result<T> {
    match value {
        Ok(value) => Ok(value),
        Err(Error::RejectedKey) => Err(error::Error::BoatKeyRejected),
        Err(Error::InvalidSpendLimit) => Err(error::Error::InvalidInput),
        Err(Error::ConnectionNotPermitted) => Err(error::Error::Conflict),
        Err(_) => Err(error::Error::Unavailable),
    }
}

pub(super) async fn read(state: &ApiState, workspace: Uuid) -> error::Result<OverflowSettings> {
    let (view, refreshed) = if let Some(service) = &state.overflow {
        match service.view(workspace).await {
            Ok(view) => (view, true),
            Err(_) => (stored(state, workspace).await?, false),
        }
    } else {
        (stored(state, workspace).await?, false)
    };
    let settings = query(state.store.clone(), move |store| store.settings(workspace)).await?;
    let account = view.account.map(|account| BoatAccountView {
        name: account.account_name,
        connected_at: account.connected_at,
        rejected_at: account.rejected_at,
    });
    let now = state.runtime.now();
    let (year, month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };
    let next_month_at = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .ok_or(error::Error::Unavailable)?
        .and_utc();
    let limit = view
        .limit_usd
        .map(|limit| Cents::from_usd(limit).ok_or(error::Error::Unavailable))
        .transpose()?;
    let spend = if refreshed {
        let counted = view.spend.counted_usd();
        Some(BoatSpendView {
            confirmed_cents: display_cents(view.spend.confirmed_usd)?,
            unconfirmed_cents: display_cents(
                view.spend.pending_usd + view.spend.settled_bound_usd,
            )?,
            settled_bound_cents: display_cents(view.spend.settled_bound_usd)?,
            bounded: counted.is_ok(),
            limit_reached: counted
                .ok()
                .zip(view.limit_usd)
                .map(|(counted, limit)| counted >= limit),
        })
    } else {
        None
    };
    Ok(OverflowSettings {
        spend,
        available: state.overflow.is_some(),
        account,
        enabled: settings.overflow_enabled,
        wait_seconds: match settings.overflow_wait {
            OverflowWait::AtOnce => 0,
            OverflowWait::Seconds15 => 15,
            OverflowWait::Seconds30 => 30,
            OverflowWait::Seconds60 => 60,
            OverflowWait::Seconds120 => 120,
        },
        spend_limit_cents: limit,
        next_month_at,
    })
}

fn display_cents(usd: Decimal) -> error::Result<Cents> {
    Cents::from_usd(usd.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero))
        .ok_or(error::Error::Unavailable)
}

async fn stored(
    state: &ApiState,
    workspace: Uuid,
) -> error::Result<ciproxy_overflow::view::OverflowView> {
    let stored = query(state.store.clone(), move |store| {
        store.boat_runtime_snapshot(workspace)
    })
    .await?;
    result(ciproxy_overflow::view::OverflowView::from_snapshot(
        stored,
        state.runtime.now(),
    ))
}
