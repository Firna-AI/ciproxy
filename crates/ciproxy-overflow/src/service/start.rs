//! Durable acceptance of a scaler Boat action before provider dispatch.

use ciproxy_scaler::actions::Destination;
use ciproxy_store_interface::overflow::admission::ReservationOutcome;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::AccountRecord;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::api::{StartOutcome, StartRequest, WaitReason};
use crate::crypto::{KeyContext, SecretPurpose};
use crate::error::{Error, Result, ResultExt};
use crate::secrets::digest;
use crate::session::Session;
use crate::spend;

use super::OverflowService;
use super::preparation::required_budget;

impl OverflowService {
    pub(super) async fn start_request(
        &self,
        workspace_id: Uuid,
        request: StartRequest,
    ) -> Result<StartOutcome> {
        let Destination::Boat(size_id) = &request.action.destination else {
            return Err(Error::NotBoatAction);
        };
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let mut snapshot = session.call(|session| session.snapshot()).await?;
        if let Some(record) = snapshot
            .starts
            .iter()
            .find(|record| record.delivery_id == request.delivery_id)
        {
            if record.runner_id != request.runner_id
                || record.organization_id != request.organization_id
                || record.label_id != request.label_id
                || record.scale_set_id != request.scale_set_id
                || record.size.name != size_id.0
                || record.requested_vcpus != request.action.requested_size.vcpu
                || u64::from(record.requested_memory_gb) * 1_000_000_000
                    != request.action.requested_size.memory_bytes
                || u64::from(record.requested_disk_gb) * 1_000_000_000
                    != request.action.requested_size.disk_bytes
            {
                return Err(Error::DeliveryConflict);
            }
            return self.execute(&session, workspace_id, record.clone()).await;
        }
        let Some(mut account) = snapshot
            .accounts
            .iter()
            .find(|account| {
                account.account.retired_at.is_none() && account.account.rejected_at.is_none()
            })
            .cloned()
        else {
            return Ok(StartOutcome::Waiting(WaitReason::Disconnected));
        };
        if !snapshot.enabled {
            return Ok(StartOutcome::Waiting(WaitReason::Disconnected));
        }
        let backend = match self
            .refresh_catalog(&session, workspace_id, &mut account)
            .await
        {
            Ok(backend) => backend,
            Err(Error::RejectedKey) => return Ok(StartOutcome::Waiting(WaitReason::Disconnected)),
            Err(error) => return Err(error),
        };
        self.refresh_costs(&session, workspace_id, &mut snapshot)
            .await?;
        let now = self.dependencies.clock.now();
        let spend = spend::current(&snapshot.starts, now)?;
        let counted = spend.counted_usd()?;
        let limit = snapshot
            .spend_limit_usd_micros
            .map(|limit| Decimal::new(limit, 6));
        if limit.is_none_or(|limit| counted >= limit) {
            return Ok(StartOutcome::Waiting(WaitReason::SpendLimit {
                resumes_at: spend::next_month(now)?,
            }));
        }
        if let Some(at) = account.state.retry_at.filter(|at| *at > now) {
            return Ok(StartOutcome::Waiting(WaitReason::RetryAt(at)));
        }
        let facts = match backend.account.account().await {
            Ok(facts) => facts,
            Err(Error::RejectedKey) => {
                account.account.rejected_at = Some(now);
                session
                    .call(move |session| session.save_account(&account))
                    .await?;
                return Ok(StartOutcome::Waiting(WaitReason::Disconnected));
            }
            Err(error) => return Err(error),
        };
        account.state.trial = facts.trial;
        if facts.billing_ready {
            account.state.billing_required = false;
        }
        let saved = account.clone();
        session
            .call(move |session| session.save_account(&saved))
            .await?;
        if account.state.billing_required {
            return Ok(StartOutcome::Waiting(WaitReason::BillingRequired));
        }
        let Some(size) = account
            .state
            .sizes
            .iter()
            .find(|size| {
                size.name == size_id.0
                    && !account.state.refused_sizes.contains(&size.name)
                    && size.vcpus == request.action.size.vcpu
                    && size.memory_bytes == request.action.size.memory_bytes
                    && size.disk_bytes == request.action.size.disk_bytes
            })
            .cloned()
        else {
            return Ok(StartOutcome::Waiting(WaitReason::SizeUnavailable));
        };
        let token = self.dependencies.tokens.token();
        let input = self.launch_input(&request, token.clone())?;
        let ciphertext = self
            .dependencies
            .cipher
            .seal(
                KeyContext {
                    workspace_id,
                    id: request.delivery_id,
                    purpose: SecretPurpose::Launch,
                },
                input,
            )
            .await?;
        let mut record = StartRecord {
            delivery_id: request.delivery_id,
            runner_id: request.runner_id,
            organization_id: request.organization_id,
            label_id: request.label_id,
            scale_set_id: request.scale_set_id,
            requested_vcpus: request.action.requested_size.vcpu,
            requested_memory_gb: u32::try_from(
                request.action.requested_size.memory_bytes / 1_000_000_000,
            )
            .internal()?,
            requested_disk_gb: u32::try_from(
                request.action.requested_size.disk_bytes / 1_000_000_000,
            )
            .internal()?,
            size,
            key_id: account.account.id,
            deployment: format!("{}/{}", self.config.deployment, workspace_id),
            snapshot: self.config.shared_snapshot.clone(),
            lifetime_seconds: if account.state.trial { 7200 } else { 21600 },
            phase: StartPhase::Unsent,
            sandbox: None,
            starts_at: now,
            ends_at: None,
            billing: None,
            start_fee_bound_usd: facts.start_fee_bound_usd,
            start_usd: None,
            reported_at: None,
            runtime: Vec::new(),
            runtime_total: None,
            running_observed_at: None,
            runtime_complete: false,
        };
        record.starts_at = self.dependencies.clock.now();
        let budget = required_budget(&snapshot.starts, &record)?;
        let pending = record.clone();
        let hash = digest(&token);
        let reserved = session
            .call(move |session| {
                session.reserve_start(&pending, &EncryptedLaunch(ciphertext), &hash, &budget)
            })
            .await?;
        match reserved {
            ReservationOutcome::NotPermitted => {
                return Ok(StartOutcome::Waiting(WaitReason::Disconnected));
            }
            ReservationOutcome::SpendLimit => {
                return Ok(StartOutcome::Waiting(WaitReason::SpendLimit {
                    resumes_at: spend::next_month(record.starts_at)?,
                }));
            }
            ReservationOutcome::Reserved => (),
        }
        self.execute(&session, workspace_id, record).await
    }

    pub(super) async fn save_refusal(
        &self,
        session: &Session,
        account: &mut AccountRecord,
        record: &mut StartRecord,
        refusal: &sandbox_interface::SandboxStartRefusal,
    ) -> Result<()> {
        let now = self.dependencies.clock.now();
        match refusal {
            sandbox_interface::SandboxStartRefusal::Temporary { retry, .. } => {
                let relative = retry
                    .retry_after
                    .and_then(|delay| chrono::Duration::from_std(delay).ok())
                    .and_then(|delay| now.checked_add_signed(delay));
                account.state.retry_at = relative.into_iter().chain(retry.next_start_at).max();
            }
            sandbox_interface::SandboxStartRefusal::PlanSize { .. } => {
                if !account.state.refused_sizes.contains(&record.size.name) {
                    account.state.refused_sizes.push(record.size.name.clone());
                }
            }
            sandbox_interface::SandboxStartRefusal::Billing { .. } => {
                account.state.billing_required = true
            }
            sandbox_interface::SandboxStartRefusal::TrialAutoStopRequired => {
                account.state.trial = true
            }
        }
        record.phase = StartPhase::Refused;
        record.ends_at = Some(now);
        record.runtime_complete = true;
        let changed = account.clone();
        let record = record.clone();
        session
            .call(move |session| session.save_refusal(&changed, &record))
            .await
    }
}
