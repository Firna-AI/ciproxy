//! Retryable unsent preparation and one durable dispatch boundary.

use ciproxy_store_interface::overflow::admission::{PreparedDispatch, ReservationOutcome};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::{ProviderParameters, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::AccountRecord;
use sandbox_interface::BackendSandbox;
use uuid::Uuid;

use crate::admission::required_budget;
use crate::api::{Retirement, StartOutcome, WaitReason};
use crate::error::Result;
use crate::session::Session;
use crate::spend;

use super::OverflowService;
use super::provider_preparation::Preparation;

pub(super) enum DispatchOutcome {
    Submitted(sandbox_interface::Result<Option<BackendSandbox>>),
    Stopped(Box<StartOutcome>),
}

impl OverflowService {
    pub(super) async fn dispatch_unsent(
        &self,
        session: &Session,
        workspace_id: Uuid,
        record: &mut StartRecord,
        account: &mut AccountRecord,
    ) -> Result<DispatchOutcome> {
        let Some(authority) = self.launch_authority(session, workspace_id, record).await? else {
            return Ok(DispatchOutcome::Stopped(Box::new(
                self.cancel_unsent(session, record).await?,
            )));
        };
        self.dependencies.providers.start_image(record.provider)?;
        let lifetime_seconds = match self
            .resume_preparation(session, account, authority.capabilities())
            .await?
        {
            Preparation::Ready { lifetime_seconds } => lifetime_seconds,
            Preparation::Waiting(wait) => {
                let delivery = record.delivery_id;
                if session
                    .call(move |session| session.startup_account(delivery))
                    .await?
                    .is_none()
                {
                    return Ok(DispatchOutcome::Stopped(Box::new(
                        self.cancel_unsent(session, record).await?,
                    )));
                }
                return Ok(DispatchOutcome::Stopped(Box::new(StartOutcome::Waiting(
                    wait,
                ))));
            }
        };
        let delivery = record.delivery_id;
        let Some(facts) = session
            .call(move |session| session.dispatch_facts(delivery))
            .await?
        else {
            return Ok(DispatchOutcome::Stopped(Box::new(
                self.cancel_unsent(session, record).await?,
            )));
        };
        let snapshot = session.call(|session| session.snapshot()).await?;
        let now = self.dependencies.clock.now();
        let Some(size) = self.selected_size(&snapshot, account, record, &facts, now)? else {
            return Ok(DispatchOutcome::Stopped(Box::new(StartOutcome::Waiting(
                WaitReason::SizeUnavailable,
            ))));
        };
        let parameters = ProviderParameters {
            size,
            lifetime_seconds,
        };
        let mut candidate = record.clone();
        candidate.dispatch = Some(parameters.clone());
        candidate.starts_at = now;
        candidate.phase = StartPhase::Creating;
        let dispatch = PreparedDispatch {
            parameters,
            starts_at: candidate.starts_at,
            budget: required_budget(&snapshot.starts, &candidate)?,
        };
        let request = self.create_request(workspace_id, &candidate)?;
        let pending = record.clone();
        let saved = dispatch.clone();
        match session
            .call(move |session| session.begin_dispatch(&pending, &saved))
            .await?
        {
            ReservationOutcome::NotPermitted => {
                let delivery = record.delivery_id;
                if session
                    .call(move |session| session.startup_account(delivery))
                    .await?
                    .is_some()
                {
                    return Ok(DispatchOutcome::Stopped(Box::new(StartOutcome::Waiting(
                        WaitReason::SizeUnavailable,
                    ))));
                }
                return Ok(DispatchOutcome::Stopped(Box::new(
                    self.cancel_unsent(session, record).await?,
                )));
            }
            ReservationOutcome::SpendLimit => {
                return Ok(DispatchOutcome::Stopped(Box::new(StartOutcome::Waiting(
                    WaitReason::SpendLimit {
                        resumes_at: spend::next_month(dispatch.starts_at)?,
                    },
                ))));
            }
            ReservationOutcome::Reserved => {}
        }
        let result = authority.create(request).await;
        *record = candidate;
        if matches!(
            result,
            Err(sandbox_interface::Error::SandboxCreateNotSent { .. })
        ) {
            tracing::warn!(
                provider = record.provider.as_str(),
                "Create was rejected before dispatch"
            );
            record.phase = StartPhase::Refused;
            record.ends_at = Some(self.dependencies.clock.now());
            record.runtime_complete = true;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
            self.collect(session).await?;
            return Ok(DispatchOutcome::Stopped(Box::new(StartOutcome::Ended(
                record.clone(),
            ))));
        }
        Ok(DispatchOutcome::Submitted(result.map(Some)))
    }

    pub(super) async fn cancel_unsent(
        &self,
        session: &Session,
        record: &mut StartRecord,
    ) -> Result<StartOutcome> {
        record.phase = StartPhase::CancelledUnsent;
        record.ends_at = Some(self.dependencies.clock.now());
        record.runtime_complete = true;
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await?;
        self.collect(session).await?;
        Ok(StartOutcome::Ended(record.clone()))
    }

    pub(super) async fn cancel_withdrawn_unsent(
        &self,
        session: &Session,
        provider: OverflowProvider,
    ) -> Result<()> {
        let snapshot = session.call(|session| session.snapshot()).await?;
        for mut record in snapshot
            .starts
            .into_iter()
            .filter(|start| start.provider == provider && start.phase == StartPhase::Unsent)
        {
            let delivery = record.delivery_id;
            if session
                .call(move |session| session.startup_account(delivery))
                .await?
                .is_none()
            {
                self.cancel_unsent(session, &mut record).await?;
            }
        }
        Ok(())
    }

    pub(super) async fn cleanup_unsent(
        &self,
        session: &Session,
        workspace_id: Uuid,
    ) -> Result<bool> {
        let snapshot = session.call(|session| session.snapshot()).await?;
        let mut failed = false;
        for mut record in snapshot
            .starts
            .into_iter()
            .filter(|start| start.phase == StartPhase::Unsent)
        {
            let delivery = record.delivery_id;
            if session
                .call(move |session| session.startup_account(delivery))
                .await?
                .is_none()
            {
                self.cancel_unsent(session, &mut record).await?;
                continue;
            }
            let retirement = self
                .dependencies
                .runners
                .retire(workspace_id, record.organization_id, record.runner_id)
                .await;
            let authorized = session
                .call(move |session| session.startup_account(delivery))
                .await?
                .is_some();
            if !authorized || matches!(retirement, Ok(Retirement::Removed)) {
                self.cancel_unsent(session, &mut record).await?;
            } else {
                failed |= retirement.is_err();
            }
        }
        Ok(failed)
    }
}
