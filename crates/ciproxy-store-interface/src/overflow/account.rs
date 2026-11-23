//! Credential-free account observations used by the overflow executor.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::overflow::provider::OverflowProvider;

/// One eligible published size with its exact runtime price.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SizeOffer {
    /// The provider size identifier.
    pub name: String,
    /// Virtual CPUs.
    pub vcpus: u32,
    /// Decimal or binary memory converted to exact bytes.
    pub memory_bytes: u64,
    /// Decimal or binary disk converted to exact bytes.
    pub disk_bytes: u64,
    /// Published USD price for one hour.
    pub hourly_usd: Decimal,
}

/// Durable account facts. A capacity read does not reserve a machine.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AccountState {
    /// The provider that supplied these observations.
    #[serde(default)]
    pub provider: OverflowProvider,
    /// Boat wallet selector, either personal or an organization ID.
    pub wallet: String,
    /// Last successful five-minute catalog read.
    pub checked_at: Option<DateTime<Utc>>,
    /// Sizes offered by the provider.
    pub sizes: Vec<SizeOffer>,
    /// The account currently has a free trial.
    pub trial: bool,
    /// Latest published or quoted start fee bound in USD.
    #[serde(default)]
    pub start_fee_bound_usd: Option<Decimal>,
    /// A definitive billing refusal remains until a successful recovery check.
    pub billing_required: bool,
    /// Earliest next attempt from a supplied relative or absolute retry time.
    pub retry_at: Option<DateTime<Utc>>,
    /// Size refusals excluded until the next successful catalog refresh.
    pub refused_sizes: Vec<String>,
}
