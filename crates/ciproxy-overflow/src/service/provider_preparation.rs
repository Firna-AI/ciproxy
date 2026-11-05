//! One provider restriction policy for fresh and durably unsent requests.

use chrono::{DateTime, Duration, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::record::StartRecord;
use ciproxy_store_interface::overflow::store::AccountRecord;
use uuid::Uuid;

use crate::api::WaitReason;
use crate::error::{Error, Result};
use crate::session::Session;

use super::OverflowService;

pub(super) fn preparation_wait(
    state: &AccountState,
    name: &str,
    size: &VmSize,
    now: DateTime<Utc>,
) -> Option<WaitReason> {
    if let Some(at) = state.retry_at.filter(|at| *at > now) {
        return Some(WaitReason::RetryAt(at));
    }
    if state.billing_required {
        return Some(WaitReason::BillingRequired);
    }
    if state.refused_sizes.iter().any(|refused| refused == name)
        || !state.sizes.iter().any(|offer| {
            offer.name == name
                && offer.vcpus == size.vcpu
                && offer.memory_bytes == size.memory_bytes
                && offer.disk_bytes == size.disk_bytes
        })
    {
        return Some(WaitReason::SizeUnavailable);
    }
    None
}

impl OverflowService {
    pub(super) async fn resume_preparation(
        &self,
        session: &Session,
        workspace_id: Uuid,
        account: &mut AccountRecord,
        record: &StartRecord,
    ) -> Result<Option<WaitReason>> {
        let size = VmSize {
            vcpu: record.size.vcpus,
            memory_bytes: record.size.memory_bytes,
            disk_bytes: record.size.disk_bytes,
        };
        let now = self.dependencies.clock.now();
        if let Some(wait @ WaitReason::RetryAt(_)) =
            preparation_wait(&account.state, &record.size.name, &size, now)
        {
            return Ok(Some(wait));
        }
        let refresh = account
            .state
            .checked_at
            .is_none_or(|at| now >= at + Duration::minutes(5));
        let recovered = if refresh {
            self.refresh_catalog(session, workspace_id, account)
                .await
                .map(|_| ())
        } else if account.state.billing_required {
            let backend = self.backend(workspace_id, account).await?;
            self.refresh_start_account(session, account, &backend)
                .await
                .map(|_| ())
        } else {
            Ok(())
        };
        match recovered {
            Ok(()) => {}
            Err(Error::RejectedKey) => return Ok(Some(WaitReason::Disconnected)),
            Err(error) => return Err(error),
        }
        Ok(preparation_wait(
            &account.state,
            &record.size.name,
            &size,
            self.dependencies.clock.now(),
        ))
    }
}
