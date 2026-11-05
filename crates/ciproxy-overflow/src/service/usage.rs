//! Sandbox-only per-month provider runtime observations and price completion.

use ciproxy_store_interface::overflow::record::{RuntimeCharge, StartRecord};
use ciproxy_store_interface::overflow::store::OverflowSnapshot;
use sandbox_interface::ProviderRef;
use uuid::Uuid;

use crate::error::Result;
use crate::provider::Capabilities;
use crate::session::Session;
use crate::spend;

use super::OverflowService;

impl OverflowService {
    pub(super) async fn refresh_costs(
        &self,
        session: &Session,
        workspace_id: Uuid,
        snapshot: &mut OverflowSnapshot,
    ) -> Result<()> {
        for record in snapshot.starts.iter_mut().filter(|record| {
            record.sandbox_id.is_some() && (record.ends_at.is_none() || !record.runtime_complete)
        }) {
            let Some(account) = snapshot
                .accounts
                .iter()
                .find(|account| account.account.id == record.key_id)
            else {
                continue;
            };
            let backend = self.backend(workspace_id, account).await?;
            self.measure(session, &backend, record).await?;
        }
        Ok(())
    }

    pub(super) async fn measure(
        &self,
        session: &Session,
        backend: &Capabilities,
        record: &mut StartRecord,
    ) -> Result<bool> {
        let Some(id) = &record.sandbox_id else {
            return Ok(false);
        };
        let id = ProviderRef::new(id);
        let now = self.dependencies.clock.now();
        let Some(total) = backend.account.usage(id.clone(), None, now).await? else {
            return Ok(false);
        };
        record.starts_at = total.since;
        let mut month = spend::month_start(total.since)?;
        let mut charges = Vec::new();
        let until = record.ends_at.unwrap_or(now).min(now);
        while month < until {
            let end = spend::next_month(month)?.min(until);
            let Some(usage) = backend.account.usage(id.clone(), Some(month), end).await? else {
                return Ok(false);
            };
            charges.push(RuntimeCharge {
                month: month.date_naive(),
                usd: usage.usd,
                until: usage.until,
                running: usage.running,
            });
            month = end;
        }
        record.runtime = charges;
        record.runtime_complete = !total.running || until == now;
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await?;
        Ok(true)
    }
}
