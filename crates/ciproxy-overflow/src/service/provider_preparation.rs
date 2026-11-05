//! Current account preparation and the shared scaler resource choice.

use chrono::{DateTime, Duration, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_scaler::size_policy::{BoatChoice, select_boat_size};
use ciproxy_scaler::snapshot::NothingFits;
use ciproxy_store_interface::labels::WhenNothingFits;
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
        disk_bytes: u64::from(record.requested_disk_gb) * 1_000_000_000,
    };
    let nothing_fits = match facts.label.when_nothing_fits {
        WhenNothingFits::Wait => NothingFits::Wait,
        WhenNothingFits::LargestBoat => NothingFits::LargestBoatSize,
    };
    let choice = select_boat_size(
        sizes.iter(),
        largest,
        requested,
        record.own_machine_fits && facts.label.allow_machines,
        nothing_fits,
    );
    let Some(BoatChoice::Fitting(size) | BoatChoice::Largest(size)) = choice else {
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
        backend: &Capabilities,
    ) -> Result<Option<WaitReason>> {
        let now = self.dependencies.clock.now();
        if let Some(wait @ WaitReason::RetryAt(_)) = preparation_wait(&account.state, now) {
            return Ok(Some(wait));
        }
        let refreshed = async {
            self.refresh_start_account(session, account, backend)
                .await?;
            let now = self.dependencies.clock.now();
            if account
                .state
                .checked_at
                .is_none_or(|at| now >= at + Duration::minutes(5))
            {
                account.state.sizes =
                    catalog::offers(backend.capacity.account_capacity().await.internal()?)?;
                account.state.checked_at = Some(now);
                account.state.refused_sizes.clear();
                let saved = account.clone();
                session
                    .call(move |session| session.save_account(&saved))
                    .await?;
            }
            Ok(())
        }
        .await;
        match refreshed {
            Ok(()) => {}
            Err(Error::RejectedKey) => return Ok(Some(WaitReason::Disconnected)),
            Err(error) => return Err(error),
        }
        Ok(preparation_wait(
            &account.state,
            self.dependencies.clock.now(),
        ))
    }
}
