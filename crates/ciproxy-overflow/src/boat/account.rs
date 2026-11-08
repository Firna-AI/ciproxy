//! Read-only account validation, explicit wallet binding, and sandbox billing.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sandbox_interface::ProviderRef;

use crate::error::{Error, Result};
use crate::provider::{AccountFacts, AccountWallet, BoatAccountApi, KeyWallets, RuntimeUsage};

use super::http::{BoatReads, valid_sandbox};
use super::wire::{Identity, Limits, Usage, WalletKind, Wallets};

pub(super) struct AccountApi {
    pub(super) reads: BoatReads,
}

#[async_trait]
impl BoatAccountApi for AccountApi {
    async fn wallets(&self) -> Result<KeyWallets> {
        let identity: Identity = self
            .reads
            .unscoped_get("/me")
            .await?
            .ok_or(Error::InvalidProviderData)?;
        let wallets: Wallets = self
            .reads
            .unscoped_get("/orgs")
            .await?
            .ok_or(Error::InvalidProviderData)?;
        if !identity.ok || identity.kind != "user.info" || !wallets.ok || wallets.kind != "org.list"
        {
            return Err(Error::InvalidProviderData);
        }
        let selected_wallet = wallets
            .orgs
            .iter()
            .find(|wallet| wallet.active)
            .or_else(|| wallets.orgs.first())
            .ok_or(Error::InvalidProviderData)?
            .selector()
            .to_owned();
        let personal_name = identity
            .user
            .display_name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(identity.user.login);
        let wallets = wallets
            .orgs
            .into_iter()
            .map(|wallet| AccountWallet {
                wallet: wallet.selector().to_owned(),
                name: match wallet.kind {
                    WalletKind::Personal => personal_name.clone(),
                    WalletKind::Organization => wallet.name,
                },
            })
            .collect();
        Ok(KeyWallets {
            wallets,
            selected_wallet,
        })
    }

    async fn account(&self) -> Result<AccountFacts> {
        let identity: Identity = self
            .reads
            .get("/me", &[])
            .await?
            .ok_or(Error::InvalidProviderData)?;
        let limits: Limits = self
            .reads
            .get("/limits", &[])
            .await?
            .ok_or(Error::InvalidProviderData)?;
        let start_fee_bound_usd = limits
            .overage_prices
            .iter()
            .flat_map(|prices| &prices.start_brackets)
            .map(|bracket| bracket.dollars)
            .max()
            .or(limits.next_start_dollars);
        if !identity.ok
            || identity.kind != "user.info"
            || !limits.ok
            || limits.kind != "limits.info"
            || start_fee_bound_usd.is_some_and(|fee| fee < Decimal::ZERO)
        {
            return Err(Error::InvalidProviderData);
        }
        let name = if self.reads.wallet == "personal" {
            if limits.team_id.is_some() {
                return Err(Error::InvalidProviderData);
            }
            identity
                .user
                .display_name
                .filter(|name| !name.trim().is_empty())
                .unwrap_or(identity.user.login)
        } else {
            if limits.team_id.as_deref() != Some(&self.reads.wallet) {
                return Err(Error::InvalidProviderData);
            }
            let wallets: Wallets = self
                .reads
                .get("/orgs", &[])
                .await?
                .ok_or(Error::InvalidProviderData)?;
            if !wallets.ok || wallets.kind != "org.list" {
                return Err(Error::InvalidProviderData);
            }
            wallets
                .orgs
                .into_iter()
                .find(|org| org.id == self.reads.wallet)
                .ok_or(Error::InvalidProviderData)?
                .name
        };
        let trial = limits.sandbox_plan_key == "trial"
            || limits.access_tier.as_deref() == Some("trial")
            || limits.has_payment_history == Some(false);
        let billing_block = |reason: Option<&str>| {
            matches!(
                reason,
                Some(
                    "billing_required" | "team_member_cap_reached" | "trial_compute_limit_reached"
                )
            )
        };
        Ok(AccountFacts {
            name,
            wallet: self.reads.wallet.clone(),
            trial,
            billing_ready: limits.billing_status == "active"
                && start_fee_bound_usd.is_some()
                && !billing_block(limits.blocked_reason.as_deref())
                && !billing_block(limits.start_blocked_reason.as_deref()),
            start_fee_bound_usd,
        })
    }

    async fn usage(
        &self,
        sandbox: ProviderRef,
        since: Option<DateTime<Utc>>,
        until: DateTime<Utc>,
    ) -> Result<Option<RuntimeUsage>> {
        if !valid_sandbox(sandbox.as_str()) {
            return Err(Error::InvalidProviderData);
        }
        let mut query = vec![("until", until.to_rfc3339())];
        if let Some(since) = since {
            query.push(("since", since.to_rfc3339()));
        }
        let result: Option<Usage> = self
            .reads
            .get(&format!("/sandboxes/{}/usage", sandbox.as_str()), &query)
            .await?;
        let Some(result) = result else {
            return Ok(None);
        };
        if !result.ok
            || result.kind != "sandbox.usage"
            || result.sandbox_id != sandbox.as_str()
            || result.dollars < Decimal::ZERO
            || result.until < result.since
            || result.until > until
        {
            return Err(Error::InvalidProviderData);
        }
        Ok(Some(RuntimeUsage {
            since: result.since,
            until: result.until,
            usd: result.dollars,
            running: result.running,
        }))
    }
}
