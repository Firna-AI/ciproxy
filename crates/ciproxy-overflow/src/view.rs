//! Credential-free UTC account facts and explicit accounting completeness.

use std::time::Duration;

use chrono::{DateTime, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::boat::{BoatAccount, BoatSize, BoatSizeId, MonthlySpend, Refusal};
use ciproxy_store_interface::boat;
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::record::StartRecord;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use uuid::Uuid;

use crate::error::{Error, Result};

/// Current-month spend with missing provider facts kept explicit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spend {
    /// Runtime and quoted start costs that are known for ciproxy sandboxes only.
    pub known_usd: Decimal,
    /// Accepted starts for which the provider supplied no start-price quote.
    pub unknown_start_charges: Vec<Uuid>,
    /// Sandboxes whose runtime cannot yet be completely measured or computed.
    pub unmeasured_runtime: Vec<Uuid>,
}

impl Spend {
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
    /// Map known facts to the scaler, using the same UTC origin as its snapshot.
    ///
    /// Inspect `spend.complete()` separately. Missing fees never become known zero.
    pub fn scaler_account(&self, clock_origin: DateTime<Utc>) -> Result<BoatAccount> {
        if !self.spend.complete() {
            return Err(Error::AccountingIncomplete);
        }
        let sizes = self
            .state
            .sizes
            .iter()
            .filter(|size| !self.state.refused_sizes.contains(&size.name))
            .map(|size| {
                Ok(BoatSize {
                    id: BoatSizeId(size.name.clone()),
                    size: VmSize {
                        vcpu: size.vcpus,
                        memory_bytes: size.memory_bytes,
                        disk_bytes: size.disk_bytes,
                    },
                    price: micros(size.hourly_usd)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
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
                spent: micros(self.spend.known_usd)?,
                limit: micros(self.limit_usd.unwrap_or(Decimal::ZERO))?,
            },
            largest_size: sizes.last().map(|size| size.id.clone()),
            sizes,
            refusals,
        })
    }
}

fn micros(value: Decimal) -> Result<u64> {
    (value * Decimal::from(1_000_000))
        .floor()
        .to_u64()
        .ok_or(Error::InvalidProviderData)
}
