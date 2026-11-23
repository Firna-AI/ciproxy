//! Current account preparation and the shared scaler resource choice.

use chrono::{DateTime, Duration, Utc};
use ciproxy_protocol::resources::{GIGABYTE, VmSize};
use ciproxy_scaler::size_policy::{ProviderChoice, select_provider_size};
use ciproxy_scaler::snapshot::NothingFits;
use ciproxy_store_interface::labels::{LabelPlace, WhenNothingFits};
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::admission::DispatchFacts;
use ciproxy_store_interface::overflow::record::StartRecord;
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};

use crate::api::WaitReason;
use crate::catalog;
use crate::error::{Error, Result, ResultExt};
use crate::provider::Capabilities;
use crate::session::Session;
use crate::spend;

use super::OverflowService;

pub(super) enum Preparation {
    Ready { lifetime_seconds: u32 },
    Waiting(WaitReason),
}

pub(super) fn preparation_wait(state: &AccountState, now: DateTime<Utc>) -> Option<WaitReason> {
    if let Some(at) = state.retry_at.filter(|at| *at > now) {
        return Some(WaitReason::RetryAt(at));
    }
    state
        .billing_required
        .then_some(WaitReason::BillingRequired)
}

impl OverflowService {
    /// Recheck fit for the saved provider and use the current order for fallback.
    /// A fitting delivery keeps its provider and original image.
    pub(super) fn selected_size(
        &self,
        snapshot: &OverflowSnapshot,
        account: &AccountRecord,
        record: &StartRecord,
        facts: &DispatchFacts,
        now: DateTime<Utc>,
    ) -> Result<Option<SizeOffer>> {
        let requested = VmSize {
            vcpu: record.requested_vcpus,
            memory_bytes: u64::from(record.requested_memory_gb) * GIGABYTE,
            disk_bytes: u64::from(record.requested_disk_gb) * GIGABYTE,
        };
        let allows_fallback = facts.label.when_nothing_fits == WhenNothingFits::LargestOverflow;
        let mut has_fit = false;
        let mut fallback = None;
        for place in &facts.label.order {
            let LabelPlace::Provider(provider) = *place else {
                has_fit |= record.own_machine_fits;
                continue;
            };
            let saved_provider = provider == record.provider;
            let (candidate, required, image_root_bytes) = if saved_provider {
                (
                    account,
                    VmSize {
                        disk_bytes: record.required_root_disk_bytes,
                        ..requested
                    },
                    record.image_root_bytes,
                )
            } else {
                if !allows_fallback || has_fit {
                    continue;
                }
                let (Ok(configured), Some(candidate)) = (
                    self.dependencies.providers.get(provider),
                    snapshot.active_account(provider),
                ) else {
                    continue;
                };
                let sizing = configured.image.root_sizing;
                let Some(required) = sizing.vm_size(requested) else {
                    continue;
                };
                (candidate, required, sizing.image_root_bytes)
            };
            let sizes = match catalog::scaler_sizes(&candidate.state) {
                Ok(sizes) => sizes,
                Err(error) if saved_provider => return Err(error),
                Err(_) => continue,
            };
            let choice = select_provider_size(
                sizes.iter(),
                sizes.last().map(|size| &size.id),
                required,
                image_root_bytes,
                has_fit,
                NothingFits::LargestProviderSize,
            );
            match choice {
                Some(ProviderChoice::Fitting(size)) if saved_provider => {
                    return Ok(candidate
                        .state
                        .sizes
                        .iter()
                        .find(|offer| offer.name == size.id.0)
                        .cloned());
                }
                Some(ProviderChoice::Fitting(_)) => has_fit = true,
                Some(ProviderChoice::Largest(size))
                    if allows_fallback
                        && fallback.is_none()
                        && (saved_provider || peer_can_start(snapshot, candidate, now)) =>
                {
                    fallback = Some((provider, size.id.0.clone()));
                }
                _ => {}
            }
        }
        if has_fit {
            return Ok(None);
        }
        Ok(fallback
            .filter(|(provider, _)| *provider == record.provider)
            .and_then(|(_, name)| account.state.sizes.iter().find(|offer| offer.name == name))
            .cloned())
    }

    pub(super) async fn resume_preparation(
        &self,
        session: &Session,
        account: &mut AccountRecord,
        capabilities: &Capabilities,
    ) -> Result<Preparation> {
        let now = self.dependencies.clock.now();
        if let Some(wait @ WaitReason::RetryAt(_)) = preparation_wait(&account.state, now) {
            return Ok(Preparation::Waiting(wait));
        }
        let refreshed = async {
            let facts = self
                .refresh_start_account(session, account, capabilities)
                .await?;
            let now = self.dependencies.clock.now();
            if account
                .state
                .checked_at
                .is_none_or(|at| now >= at + Duration::minutes(5))
            {
                account.state.sizes =
                    catalog::offers(capabilities.capacity.account_capacity().await.internal()?)?;
                account.state.checked_at = Some(now);
                account.state.refused_sizes.clear();
                let saved = account.clone();
                session
                    .call(move |session| session.save_account(&saved))
                    .await?;
            }
            Ok(facts)
        }
        .await;
        let facts = match refreshed {
            Ok(facts) => facts,
            Err(Error::RejectedKey) => return Ok(Preparation::Waiting(WaitReason::Disconnected)),
            Err(error) => return Err(error),
        };
        Ok(
            match preparation_wait(&account.state, self.dependencies.clock.now()) {
                Some(wait) => Preparation::Waiting(wait),
                None => Preparation::Ready {
                    lifetime_seconds: facts.lifetime_seconds,
                },
            },
        )
    }
}

/// Check other providers using the same holds the scaler sees.
fn peer_can_start(
    snapshot: &OverflowSnapshot,
    account: &AccountRecord,
    now: DateTime<Utc>,
) -> bool {
    let provider = account.account.provider;
    account.account.rejected_at.is_none()
        && preparation_wait(&account.state, now).is_none()
        && snapshot
            .provider_settings
            .get(&provider)
            .and_then(|settings| settings.spend_limit_usd_micros)
            .and_then(|limit| u64::try_from(limit).ok())
            .is_some_and(|limit| {
                spend::current(provider, &snapshot.starts, now)
                    .and_then(|spend| catalog::micros(spend.counted_usd()?))
                    .is_ok_and(|counted| counted < limit)
            })
}
