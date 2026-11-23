//! Provider Setup and Overflow Settings requests with no credential in responses.

use chrono::{DateTime, Utc};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::provider;
use serde::{Deserialize, Serialize};
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
use utoipa::{IntoParams, PartialSchema, ToSchema};

use crate::access::workspace::WorkspaceId;
use crate::wire::money::Cents;

/// The closed set of supported overflow providers, encoded as strings.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct OverflowProvider(
    /// The same provider identity used for stored accounts and runners.
    pub provider::OverflowProvider,
);

impl From<provider::OverflowProvider> for OverflowProvider {
    fn from(provider: provider::OverflowProvider) -> Self {
        Self(provider)
    }
}

impl PartialSchema for OverflowProvider {
    fn schema() -> RefOr<Schema> {
        ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some(
                provider::OverflowProvider::all().map(|id| id.as_str()),
            ))
            .into()
    }
}

impl ToSchema for OverflowProvider {}

/// A provider scoped to a workspace route.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct OverflowProviderPath {
    /// Stable workspace identity.
    #[param(value_type = uuid::Uuid)]
    pub workspace_id: WorkspaceId,
    /// Provider whose connection or settings the request changes.
    #[param(inline)]
    pub provider: OverflowProvider,
}

/// Confirmation of a change to one provider.
#[derive(Serialize, ToSchema)]
pub struct ProviderSuccess {
    /// True when the change succeeded.
    pub ok: bool,
    /// Provider whose change succeeded.
    pub provider: OverflowProvider,
}

/// One-use key check before the customer selects a Boat wallet.
#[derive(Deserialize, ToSchema)]
pub struct BoatKeyCheck {
    /// The customer's Boat key. Wallet discovery does not save it.
    #[schema(value_type=String, write_only=true)]
    pub api_key: Secret,
}

/// A named billing wallet that the checked key can use.
#[derive(Serialize, ToSchema)]
pub struct BoatWallet {
    /// Explicit personal selector or stable Boat organization identity.
    pub wallet: String,
    /// The verified personal account or organization name.
    pub name: String,
}

/// Safe key-check result with the likely wallet selected.
#[derive(Serialize, ToSchema)]
pub struct BoatWallets {
    /// Provider that accepted the key and returned these wallets.
    pub provider: OverflowProvider,
    /// All wallets accessible to the key.
    pub wallets: Vec<BoatWallet>,
    /// Boat's active wallet, or the first offered wallet when none is active.
    pub selected_wallet: String,
}

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
    /// Whether the workspace permits new overflow runners.
    pub enabled: bool,
    /// The supported overflow waits are 0, 15, 30, 60, and 120 seconds.
    pub wait_seconds: u16,
}

/// A monthly spend limit for the provider named in the path.
#[derive(Deserialize, ToSchema)]
pub struct ProviderSpendLimit {
    /// Positive monthly integer US cents as a decimal string.
    pub spend_limit_cents: Cents,
}

/// Safe connected-account metadata with no key or key ciphertext.
#[derive(Serialize, ToSchema)]
pub struct ProviderAccountView {
    /// The account name returned by the provider during verification.
    pub name: String,
    /// The time this key version connected.
    pub connected_at: DateTime<Utc>,
    /// The first time the provider refused this key, absent while the key is accepted.
    pub rejected_at: Option<DateTime<Utc>>,
}

/// Workspace routing policy and provider options for Setup and Settings.
#[derive(Serialize, ToSchema)]
pub struct OverflowSettings {
    /// The switch that permits new overflow starts.
    pub enabled: bool,
    /// The saved overflow wait in seconds.
    pub wait_seconds: u16,
    /// First instant of the next calendar month in UTC.
    pub next_month_at: DateTime<Utc>,
    /// Every supported provider, including unavailable or disconnected providers.
    pub providers: Vec<ProviderSettings>,
}

/// Deployment and workspace facts for one supported provider.
#[derive(Serialize, ToSchema)]
pub struct ProviderSettings {
    /// Stable provider identity.
    pub provider: OverflowProvider,
    /// Provider brand name.
    pub name: &'static str,
    /// The deployment has the configuration needed to offer this provider.
    pub available: bool,
    /// The key can use several billing accounts and the customer selects one.
    pub has_billing_account_choice: bool,
    /// The workspace can add this provider to its Linux label orders.
    pub offered: bool,
    /// Safe account metadata, absent after skip or disconnect.
    pub account: Option<ProviderAccountView>,
    /// Current-month display, absent when spend observations are unavailable.
    pub spend: Option<ProviderSpendView>,
    /// Saved limit for the connection or reconnect form.
    pub spend_limit_cents: Option<Cents>,
}

/// Rounded display amounts. Exact decimal costs still govern admission.
#[derive(Serialize, ToSchema)]
pub struct ProviderSpendView {
    /// Provider-confirmed costs, rounded once after aggregation to integer US cents.
    pub confirmed_cents: Cents,
    /// Unconfirmed costs, including pending bounds and expired settled bounds.
    /// Rounded once after both parts are added. This is not a provider-confirmed charge.
    pub unconfirmed_cents: Cents,
    /// The settled-bound portion of unconfirmed costs, rounded after aggregation.
    pub settled_bound_cents: Cents,
    /// Whether all current-month costs have either a known value or a safe bound.
    pub bounded: bool,
    /// Whether the saved limit cannot cover the cheapest eligible start.
    pub limit_reached: Option<bool>,
}
