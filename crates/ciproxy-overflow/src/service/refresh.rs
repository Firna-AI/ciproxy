//! Five-minute catalog refresh, key rejection, billing recovery, and safe views.

use chrono::Duration;
use ciproxy_store_interface::overflow::store::{AccountRecord, OverflowSnapshot};
use uuid::Uuid;

use crate::catalog;
use crate::error::{Error, Result, ResultExt};
use crate::provider::{AccountFacts, Capabilities};
use crate::session::Session;
use crate::view::OverflowView;

use super::OverflowService;

impl OverflowService {
    pub(super) async fn refresh_start_account(
        &self,
        session: &Session,
        account: &mut AccountRecord,
        backend: &Capabilities,
    ) -> Result<AccountFacts> {
        let facts = match backend.account.account().await {
            Ok(facts) => facts,
            Err(Error::RejectedKey) => {
                account.account.rejected_at = Some(self.dependencies.clock.now());
                let saved = account.clone();
                session
                    .call(move |session| session.save_account(&saved))
                    .await?;
                return Err(Error::RejectedKey);
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
        Ok(facts)
    }

    pub(super) async fn refresh_catalog(
        &self,
        session: &Session,
        workspace_id: Uuid,
        account: &mut AccountRecord,
    ) -> Result<Capabilities> {
        let backend = self.backend(workspace_id, account).await?;
        let now = self.dependencies.clock.now();
        if account
            .state
            .checked_at
            .is_none_or(|at| now >= at + Duration::minutes(5))
        {
            let facts = match backend.account.account().await {
                Ok(facts) => facts,
                Err(Error::RejectedKey) => {
                    account.account.rejected_at = Some(now);
                    let changed = account.clone();
                    session
                        .call(move |session| session.save_account(&changed))
                        .await?;
                    return Err(Error::RejectedKey);
                }
                Err(error) => return Err(error),
            };
            account.state.sizes =
                catalog::offers(backend.capacity.account_capacity().await.internal()?)?;
            account.state.checked_at = Some(now);
            account.state.trial = facts.trial;
            account.state.refused_sizes.clear();
            if facts.billing_ready {
                account.state.billing_required = false;
            }
            let changed = account.clone();
            session
                .call(move |session| session.save_account(&changed))
                .await?;
        }
        Ok(backend)
    }

    pub(super) async fn read_view(&self, workspace_id: Uuid) -> Result<OverflowView> {
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let mut snapshot = session.call(|session| session.snapshot()).await?;
        if let Some(account) = snapshot.accounts.iter_mut().find(|account| {
            account.account.retired_at.is_none() && account.account.rejected_at.is_none()
        }) {
            match self.refresh_catalog(&session, workspace_id, account).await {
                Ok(_) | Err(Error::RejectedKey) => {}
                Err(error) => return Err(error),
            }
        }
        self.refresh_costs(&session, workspace_id, &mut snapshot)
            .await?;
        self.make_view(snapshot)
    }

    pub(super) fn make_view(&self, snapshot: OverflowSnapshot) -> Result<OverflowView> {
        OverflowView::from_snapshot(snapshot, self.dependencies.clock.now())
    }
}
