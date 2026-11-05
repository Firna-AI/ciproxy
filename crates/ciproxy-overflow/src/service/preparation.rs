//! Retryable unsent preparation and one durable dispatch boundary.

use ciproxy_store_interface::overflow::admission::{
    PreparedDispatch, ReservationBudget, ReservationOutcome,
};
use ciproxy_store_interface::overflow::record::{ProviderParameters, StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::AccountRecord;
use sandbox_interface::BackendSandbox;
use uuid::Uuid;

use crate::api::{Retirement, StartOutcome, WaitReason};
use crate::budget;
use crate::error::Result;
use crate::session::Session;
use crate::spend;

use super::OverflowService;
use super::execution::create_request;
use super::provider_preparation::selected_size;

pub(super) enum DispatchOutcome {
    Submitted(sandbox_interface::Result<Option<BackendSandbox>>),
    Stopped(Box<StartOutcome>),
}

/// Count the selected dispatch once alongside every accepted provider bound.
pub(super) fn required_budget(
    starts: &[StartRecord],
    record: &StartRecord,
) -> Result<ReservationBudget> {
    let mut starts: Vec<_> = starts
        .iter()
        .filter(|start| start.delivery_id != record.delivery_id)
        .cloned()
        .collect();
    let mut dispatched = record.clone();
    dispatched.phase = StartPhase::Creating;
    starts.push(dispatched);
    let next_month = spend::next_month(record.starts_at)?;
    Ok(ReservationBudget {
        current_month_usd: spend::current(&starts, record.starts_at)?.counted_usd()?,
        next_month_usd: if next_month < budget::expires_at(record)? {
            Some(spend::current(&starts, next_month)?.counted_usd()?)
        } else {
            None
        },
    })
}

impl OverflowService {
    pub(super) async fn dispatch_unsent(
        &self,
        session: &Session,
        workspace_id: Uuid,
        record: &mut StartRecord,
        account: &mut AccountRecord,
    ) -> Result<DispatchOutcome> {
        let Some(authority) = self
            .launch_authority(session, workspace_id, record.delivery_id)
            .await?
        else {
            return Ok(DispatchOutcome::Stopped(Box::new(
                self.cancel_unsent(session, record).await?,
            )));
        };
        if let Some(wait) = self
            .resume_preparation(session, account, authority.backend())
            .await?
        {
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
        let delivery = record.delivery_id;
        let Some(facts) = session
            .call(move |session| session.dispatch_facts(delivery))
            .await?
        else {
            return Ok(DispatchOutcome::Stopped(Box::new(
                self.cancel_unsent(session, record).await?,
            )));
        };
        let Some(size) = selected_size(&account.state, record, &facts)? else {
            return Ok(DispatchOutcome::Stopped(Box::new(StartOutcome::Waiting(
                WaitReason::SizeUnavailable,
            ))));
        };
        let parameters = ProviderParameters {
            size,
            lifetime_seconds: if account.state.trial { 7200 } else { 21600 },
        };
        let mut candidate = record.clone();
        candidate.dispatch = Some(parameters.clone());
        candidate.starts_at = self.dependencies.clock.now();
        candidate.phase = StartPhase::Creating;
        let snapshot = session.call(|session| session.snapshot()).await?;
        let dispatch = PreparedDispatch {
            parameters,
            starts_at: candidate.starts_at,
            budget: required_budget(&snapshot.starts, &candidate)?,
        };
        let request = create_request(workspace_id, &candidate)?;
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
        let result = authority.create(request).await.map(Some);
        *record = candidate;
        Ok(DispatchOutcome::Submitted(result))
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

    pub(super) async fn cancel_withdrawn_unsent(&self, session: &Session) -> Result<()> {
        let snapshot = session.call(|session| session.snapshot()).await?;
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
