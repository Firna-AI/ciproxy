//! Credential-free UTC account facts and explicit accounting completeness.

use std::time::Duration;

use chrono::{DateTime, Utc};
use ciproxy_scaler::provider::{AccountFacts, MonthlySpend, Refusal};
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::StartRecord;
use ciproxy_store_interface::overflow::store::OverflowSnapshot;
use ciproxy_store_interface::overflow_provider_accounts::OverflowProviderAccount;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::catalog::{ResourceCatalog, micros, resources, scaler_sizes};
use crate::error::{Error, Result};
use crate::spend;

/// Current-month spend with missing provider facts kept explicit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spend {
    /// Measured runtime and provider-confirmed actual start costs.
    pub confirmed_usd: Decimal,
    /// Unconfirmed costs covered by the remaining full-lifetime bounds.
    pub pending_usd: Decimal,
    /// Expired budget bounds, without a claim of provider confirmation.
    pub settled_bound_usd: Decimal,
    /// Deliveries whose required costs have neither facts nor a safe bound.
    pub unbounded_starts: Vec<Uuid>,
    /// Starts whose actual fee remains unconfirmed.
    pub unknown_start_charges: Vec<Uuid>,
    /// Sandboxes whose runtime cannot yet be completely measured or computed.
    pub unmeasured_runtime: Vec<Uuid>,
}

impl Spend {
    /// Amount counted against the limit, or an explicit missing-bound error.
    pub fn counted_usd(&self) -> Result<Decimal> {
        if !self.unbounded_starts.is_empty() {
            return Err(Error::AccountingIncomplete);
        }
        Ok(self.confirmed_usd + self.pending_usd + self.settled_bound_usd)
    }

    /// Whether all starts and runtime intervals of this month have a cost.
    pub fn complete(&self) -> bool {
        self.unknown_start_charges.is_empty() && self.unmeasured_runtime.is_empty()
    }
}

/// Workspace overflow facts for the snapshot builder and the dashboard API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowView {
    /// Account, spend, and catalog facts for each supported provider.
    pub providers: Vec<ProviderView>,
    /// Workspace switch for new overflow starts.
    pub enabled: bool,
    /// Accepted deliveries to include before asking the scaler for another start.
    pub starts: Vec<StartRecord>,
}

impl OverflowView {
    /// Read durable supply and costs without external provider access.
    pub fn from_snapshot(snapshot: OverflowSnapshot, now: DateTime<Utc>) -> Result<Self> {
        Self::with_refreshes(
            snapshot,
            now,
            OverflowProvider::all().map(|provider| (provider, RefreshStatus::Stored)),
        )
    }

    pub(crate) fn with_refreshes(
        snapshot: OverflowSnapshot,
        now: DateTime<Utc>,
        refreshes: impl IntoIterator<Item = (OverflowProvider, RefreshStatus)>,
    ) -> Result<Self> {
        let providers = refreshes
            .into_iter()
            .map(|(provider, refresh)| {
                ProviderView::from_snapshot(provider, &snapshot, now, refresh)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            providers,
            enabled: snapshot.enabled,
            starts: snapshot.starts,
        })
    }

    /// Read the view of one provider.
    pub fn provider(&self, provider: OverflowProvider) -> Option<&ProviderView> {
        self.providers.iter().find(|view| view.provider == provider)
    }
}

/// How the service obtained one provider's observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshStatus {
    /// The caller supplied a durable snapshot without a deployment check.
    Stored,
    /// The provider is configured and its refresh completed.
    Current,
    /// The deployment lacks the provider's configuration.
    Unavailable,
    /// A configured provider's refresh failed, so its observations are saved facts.
    Failed,
}

/// Credential-free facts and saved settings for one provider of a workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderView {
    /// The provider whose account and spend this view describes.
    pub provider: OverflowProvider,
    /// Current accepted or rejected account metadata. No ciphertext is exposed.
    pub account: Option<OverflowProviderAccount>,
    /// Current catalog and durable refusal facts.
    pub state: AccountState,
    /// Current UTC-month cost on this provider across all key versions and organizations.
    pub spend: Spend,
    /// Saved positive monthly limit in USD.
    pub limit_usd: Option<Decimal>,
    /// Whether the workspace has offered this provider to its Linux labels.
    pub offered: bool,
    /// Availability and freshness of provider observations.
    pub refresh: RefreshStatus,
}

impl ProviderView {
    /// Return physical account fit without requiring costs or start availability.
    pub fn resource_catalog(&self) -> Result<Option<ResourceCatalog>> {
        if self.account.is_none() || self.state.checked_at.is_none() {
            return Ok(None);
        }
        resources(&self.state).map(Some)
    }

    fn from_snapshot(
        provider: OverflowProvider,
        snapshot: &OverflowSnapshot,
        now: DateTime<Utc>,
        refresh: RefreshStatus,
    ) -> Result<Self> {
        let account = snapshot.active_account(provider);
        let settings = snapshot.provider_settings.get(&provider);
        Ok(Self {
            provider,
            account: account.map(|account| account.account.clone()),
            state: account.map_or_else(
                || AccountState {
                    provider,
                    ..AccountState::default()
                },
                |account| account.state.clone(),
            ),
            spend: spend::current(provider, &snapshot.starts, now)?,
            limit_usd: settings
                .and_then(|settings| settings.spend_limit_usd_micros)
                .map(|limit| Decimal::new(limit, 6)),
            offered: settings.is_some_and(|settings| settings.offered),
            refresh,
        })
    }

    /// Preserve catalog policy for retained runners while permitting no new start.
    pub fn retained_scaler_account(&self) -> Result<AccountFacts> {
        let sizes = scaler_sizes(&self.state)?;
        Ok(AccountFacts {
            connected: self.account.is_some(),
            spend: MonthlySpend { spent: 0, limit: 0 },
            largest_size: sizes.last().map(|size| size.id.clone()),
            sizes,
            refusals: Vec::new(),
        })
    }
    /// Map known facts to the scaler, using the same UTC origin as its snapshot.
    ///
    /// Pending and settled bounds count without becoming measured costs.
    pub fn scaler_account(&self, clock_origin: DateTime<Utc>) -> Result<AccountFacts> {
        if matches!(
            self.refresh,
            RefreshStatus::Unavailable | RefreshStatus::Failed
        ) {
            return Err(Error::ProviderUnavailable);
        }
        let counted = self.spend.counted_usd()?;
        let sizes = scaler_sizes(&self.state)?;
        let mut refusals = Vec::new();
        if self
            .account
            .as_ref()
            .is_some_and(|account| account.rejected_at.is_some())
        {
            refusals.push(Refusal::RejectedKey);
        }
        if self.state.billing_required {
            refusals.push(Refusal::BillingRequired);
        }
        if let Some(at) = self.state.retry_at {
            let delay = (at - clock_origin).to_std().unwrap_or(Duration::ZERO);
            refusals.push(Refusal::RetryAfter { not_before: delay });
        }
        Ok(AccountFacts {
            connected: self.account.is_some(),
            spend: MonthlySpend {
                spent: micros(counted)?,
                limit: micros(self.limit_usd.unwrap_or(Decimal::ZERO))?,
            },
            largest_size: sizes.last().map(|size| size.id.clone()),
            sizes,
            refusals,
        })
    }
}
