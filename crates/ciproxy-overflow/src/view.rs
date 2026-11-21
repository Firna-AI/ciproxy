//! Credential-free UTC account facts and explicit accounting completeness.

use std::time::Duration;

use chrono::{DateTime, Utc};
use ciproxy_scaler::boat::{BoatAccount, MonthlySpend, Refusal};
use ciproxy_store_interface::boat;
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::record::StartRecord;
use ciproxy_store_interface::overflow::store::OverflowSnapshot;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::catalog::{ResourceCatalog, micros, resources, scaler_sizes};
use crate::error::{Error, Result};

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

/// Overflow facts for the snapshot builder and the dashboard API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowView {
    /// Current accepted or rejected account metadata. No ciphertext is exposed.
    pub account: Option<boat::BoatAccount>,
    /// Current catalog and durable refusal facts.
    pub state: AccountState,
    /// Current UTC-month cost, including old keys and other organizations.
    pub spend: Spend,
    /// Saved positive monthly limit in USD.
    pub limit_usd: Option<Decimal>,
    /// Workspace switch for new Boat starts.
    pub enabled: bool,
    /// Accepted deliveries to include before asking the scaler for another start.
    pub starts: Vec<StartRecord>,
}

impl OverflowView {
    /// Return physical account fit without requiring costs or start availability.
    pub fn resource_catalog(&self) -> Result<Option<ResourceCatalog>> {
        if self.account.is_none() || self.state.checked_at.is_none() {
            return Ok(None);
        }
        resources(&self.state).map(Some)
    }

    /// Read durable supply and costs without external provider access.
    pub fn from_snapshot(snapshot: OverflowSnapshot, now: DateTime<Utc>) -> Result<Self> {
        let account = snapshot
            .accounts
            .into_iter()
            .find(|account| account.account.retired_at.is_none());
        Ok(Self {
            account: account.as_ref().map(|account| account.account.clone()),
            state: account.map(|account| account.state).unwrap_or_default(),
            spend: crate::spend::current(&snapshot.starts, now)?,
            limit_usd: snapshot
                .spend_limit_usd_micros
                .map(|limit| Decimal::new(limit, 6)),
            enabled: snapshot.enabled,
            starts: snapshot.starts,
        })
    }

    /// Preserve catalog policy for retained runners while permitting no new start.
    pub fn retained_scaler_account(&self) -> Result<BoatAccount> {
        let sizes = scaler_sizes(&self.state)?;
        Ok(BoatAccount {
            enabled: false,
            connected: false,
            spend: MonthlySpend { spent: 0, limit: 0 },
            largest_size: sizes.last().map(|size| size.id.clone()),
            sizes,
            refusals: Vec::new(),
        })
    }
    /// Map known facts to the scaler, using the same UTC origin as its snapshot.
    ///
    /// Pending and settled bounds count without becoming measured costs.
    pub fn scaler_account(&self, clock_origin: DateTime<Utc>) -> Result<BoatAccount> {
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
        Ok(BoatAccount {
            enabled: self.enabled,
            connected: self
                .account
                .as_ref()
                .is_some_and(|account| account.rejected_at.is_none()),
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
