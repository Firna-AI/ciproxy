//! Typed minimal Boat account, wallet, limits, and runtime JSON contracts.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Identity {
    pub(super) ok: bool,
    #[serde(rename = "type")]
    pub(super) kind: String,
    pub(super) user: User,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct User {
    pub(super) login: String,
    pub(super) display_name: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Wallets {
    pub(super) ok: bool,
    #[serde(rename = "type")]
    pub(super) kind: String,
    pub(super) orgs: Vec<Wallet>,
}

#[derive(Deserialize)]
pub(super) struct Wallet {
    pub(super) id: String,
    pub(super) name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Limits {
    pub(super) ok: bool,
    #[serde(rename = "type")]
    pub(super) kind: String,
    pub(super) team_id: Option<String>,
    pub(super) sandbox_plan_key: String,
    pub(super) access_tier: Option<String>,
    pub(super) has_payment_history: Option<bool>,
    pub(super) billing_status: String,
    pub(super) blocked_reason: Option<String>,
    pub(super) start_blocked_reason: Option<String>,
    #[serde(default, with = "rust_decimal::serde::arbitrary_precision_option")]
    pub(super) next_start_dollars: Option<Decimal>,
    pub(super) overage_prices: Option<OveragePrices>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OveragePrices {
    #[serde(default)]
    pub(super) start_brackets: Vec<StartBracket>,
}

#[derive(Deserialize)]
pub(super) struct StartBracket {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub(super) dollars: Decimal,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Usage {
    pub(super) ok: bool,
    #[serde(rename = "type")]
    pub(super) kind: String,
    pub(super) sandbox_id: String,
    pub(super) since: DateTime<Utc>,
    pub(super) until: DateTime<Utc>,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub(super) dollars: Decimal,
    pub(super) running: bool,
}
