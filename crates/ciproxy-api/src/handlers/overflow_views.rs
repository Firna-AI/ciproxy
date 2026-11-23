//! Credential-free provider observations for Settings, Labels, and Jobs.

use chrono::{DateTime, Datelike, Months, NaiveTime, Utc};
use ciproxy_overflow::view::{OverflowView, ProviderView, RefreshStatus};
use ciproxy_store_interface::settings::OverflowWait;
use rust_decimal::{Decimal, RoundingStrategy};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::pages::PageServices;
use crate::state::{ApiState, query};
use crate::wire::money::Cents;
use crate::wire::overflow::{
    OverflowSettings, ProviderAccountView, ProviderSettings, ProviderSpendView,
};

pub(super) async fn observe(
    state: &ApiState,
    pages: Option<&PageServices>,
    workspace: Uuid,
) -> Option<OverflowView> {
    let service = pages
        .and_then(|pages| pages.overflow.as_deref())
        .or(state.overflow.as_deref())?;
    service.view(workspace).await.ok()
}

pub(super) async fn read(
    state: &ApiState,
    pages: Option<&PageServices>,
    workspace: Uuid,
) -> Result<OverflowSettings> {
    let now = state.runtime.now();
    let observed = observe(state, pages, workspace).await;
    let view = match observed {
        Some(view) => view,
        None => {
            let snapshot = query(state.store.clone(), move |store| {
                store.overflow_runtime_snapshot(workspace)
            })
            .await?;
            OverflowView::from_snapshot(snapshot, now).map_err(|_| Error::Unavailable)?
        }
    };
    let settings = query(state.store.clone(), move |store| store.settings(workspace)).await?;
    Ok(OverflowSettings {
        providers: view
            .providers
            .iter()
            .map(|view| provider_settings(view, pages))
            .collect::<Result<_>>()?,
        enabled: settings.overflow_enabled,
        wait_seconds: wait_seconds(settings.overflow_wait),
        next_month_at: next_month(now)?,
    })
}

fn provider_settings(
    view: &ProviderView,
    pages: Option<&PageServices>,
) -> Result<ProviderSettings> {
    let descriptor = view.provider.descriptor();
    let available = match view.refresh {
        RefreshStatus::Current | RefreshStatus::Failed => true,
        RefreshStatus::Unavailable => false,
        RefreshStatus::Stored => {
            pages.is_some_and(|pages| pages.provider_roots.contains_key(&view.provider))
        }
    };
    let account = view.account.as_ref().map(|account| ProviderAccountView {
        name: account.account_name.clone(),
        connected_at: account.connected_at,
        rejected_at: account.rejected_at,
    });
    let spend_limit_cents = view
        .limit_usd
        .map(|limit| Cents::from_usd(limit).ok_or(Error::Unavailable))
        .transpose()?;
    let spend = match view.refresh {
        RefreshStatus::Current => Some(spend_view(view)?),
        RefreshStatus::Unavailable | RefreshStatus::Stored if account.is_some() => {
            Some(spend_view(view)?)
        }
        _ => None,
    };
    Ok(ProviderSettings {
        provider: view.provider.into(),
        name: descriptor.display_name,
        available,
        has_billing_account_choice: descriptor.billing_account_kind.is_some(),
        offered: view.offered,
        account,
        spend,
        spend_limit_cents,
    })
}

fn spend_view(view: &ProviderView) -> Result<ProviderSpendView> {
    let counted = view.spend.counted_usd();
    Ok(ProviderSpendView {
        confirmed_cents: display_cents(view.spend.confirmed_usd)?,
        unconfirmed_cents: display_cents(view.spend.pending_usd + view.spend.settled_bound_usd)?,
        settled_bound_cents: display_cents(view.spend.settled_bound_usd)?,
        bounded: counted.is_ok(),
        limit_reached: view.limit_reached(),
    })
}

fn display_cents(usd: Decimal) -> Result<Cents> {
    Cents::from_usd(usd.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero))
        .ok_or(Error::Unavailable)
}

pub(super) fn next_month(at: DateTime<Utc>) -> Result<DateTime<Utc>> {
    Ok(at
        .date_naive()
        .with_day(1)
        .and_then(|date| date.checked_add_months(Months::new(1)))
        .ok_or(Error::Unavailable)?
        .and_time(NaiveTime::MIN)
        .and_utc())
}

pub(super) fn wait_seconds(wait: OverflowWait) -> u16 {
    match wait {
        OverflowWait::AtOnce => 0,
        OverflowWait::Seconds15 => 15,
        OverflowWait::Seconds30 => 30,
        OverflowWait::Seconds60 => 60,
        OverflowWait::Seconds120 => 120,
    }
}
