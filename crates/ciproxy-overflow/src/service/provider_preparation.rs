//! Current account preparation and the shared scaler resource choice.

use chrono::{DateTime, Duration, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::size_policy::{ProviderChoice, select_provider_size};
use ciproxy_scaler::snapshot::NothingFits;
use ciproxy_store_interface::labels::{LabelPlace, WhenNothingFits};
use ciproxy_store_interface::overflow::account::{AccountState, SizeOffer};
use ciproxy_store_interface::overflow::admission::DispatchFacts;
use ciproxy_store_interface::overflow::record::StartRecord;
use ciproxy_store_interface::overflow::store::AccountRecord;

use crate::api::WaitReason;
use crate::catalog;
use crate::error::{Error, Result, ResultExt};
use crate::provider::Capabilities;
use crate::session::Session;

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

/// Replan Boat capacity with the scaler's retained own-machine admission context.
/// Later own-machine capacity changes remain the scaler's responsibility.
pub(super) fn selected_size(
    state: &AccountState,
    record: &StartRecord,
    facts: &DispatchFacts,
) -> Result<Option<SizeOffer>> {
    let sizes = catalog::scaler_sizes(state)?;
    let largest = sizes.last().map(|size| &size.id);
    let requested = VmSize {
        vcpu: record.requested_vcpus,
        memory_bytes: u64::from(record.requested_memory_gb) * 1_000_000_000,
        disk_bytes: record.required_root_disk_bytes,
    };
    let nothing_fits = match facts.label.when_nothing_fits {
        WhenNothingFits::Wait => NothingFits::Wait,
        WhenNothingFits::LargestOverflow => NothingFits::LargestProviderSize,
    };
    let choice = select_provider_size(
        sizes.iter(),
        largest,
        requested,
        record.image_root_bytes,
        record.own_machine_fits && facts.label.order.contains(&LabelPlace::Machine),
        nothing_fits,
    );
    let Some(ProviderChoice::Fitting(size) | ProviderChoice::Largest(size)) = choice else {
        return Ok(None);
    };
    Ok(state
        .sizes
        .iter()
        .find(|offer| offer.name == size.id.0)
        .cloned())
}

impl OverflowService {
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
