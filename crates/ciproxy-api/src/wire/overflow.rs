//! Boat Setup and Overflow Settings requests with no credential in responses.

use chrono::{DateTime, Utc};
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::wire::money::Cents;

/// Key verification and a required monthly limit for every Boat connection.
#[derive(Deserialize, ToSchema)]
pub struct BoatConnection {
    /// The customer's own Boat key, used only for this request.
    #[schema(value_type=String, write_only=true)]
    pub api_key: Secret,
    /// Boat wallet identity. Personal is the default when no organization is selected.
    #[serde(default = "personal")]
    pub wallet: String,
    /// Positive integer US cents as a decimal string.
    pub spend_limit_cents: Cents,
}

fn personal() -> String {
    "personal".to_owned()
}

/// Complete routing policy editable in Settings under Overflow.
#[derive(Deserialize, ToSchema)]
pub struct OverflowPolicy {
    /// Whether the workspace permits new Boat runners.
    pub enabled: bool,
    /// The supported overflow waits are 0, 15, 30, 60, and 120 seconds.
    pub wait_seconds: u16,
    /// Positive monthly integer US cents as a decimal string.
    pub spend_limit_cents: Cents,
}

/// Safe connected-account metadata with no key or key ciphertext.
#[derive(Serialize, ToSchema)]
pub struct BoatAccountView {
    /// The account name returned by Boat during verification.
    pub name: String,
    /// The time this key version connected.
    pub connected_at: DateTime<Utc>,
    /// The first time Boat refused this key, absent while the key is accepted.
    pub rejected_at: Option<DateTime<Utc>>,
}

/// Boat Setup and Settings data shared by their connection form.
#[derive(Serialize, ToSchema)]
pub struct OverflowSettings {
    /// Current-month cost display, absent when provider observations are unavailable.
    pub spend: Option<BoatSpendView>,
    /// Whether the deployment has a shared Boat image and the Boat service.
    pub available: bool,
    /// Safe account metadata, absent after skip or disconnect.
    pub account: Option<BoatAccountView>,
    /// The switch that permits new Boat starts.
    pub enabled: bool,
    /// The saved overflow wait in seconds.
    pub wait_seconds: u16,
    /// Saved limit for the connection or reconnect form.
    pub spend_limit_cents: Option<Cents>,
    /// First instant of the next calendar month in UTC.
    pub next_month_at: DateTime<Utc>,
}

/// Rounded display amounts. Exact decimal costs still govern Boat admission.
#[derive(Serialize, ToSchema)]
pub struct BoatSpendView {
    /// Provider-confirmed costs, rounded once after aggregation to integer US cents.
    pub confirmed_cents: Cents,
    /// Unconfirmed costs, including pending bounds and expired settled bounds.
    /// Rounded once after both parts are added. This is not a provider-confirmed charge.
    pub unconfirmed_cents: Cents,
    /// The settled-bound portion of unconfirmed costs, rounded after aggregation.
    pub settled_bound_cents: Cents,
    /// Whether all current-month costs have either a known value or a safe bound.
    pub bounded: bool,
    /// Whether exact counted costs have reached the saved monthly limit.
    pub limit_reached: Option<bool>,
}
