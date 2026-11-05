//! Sandbox-only per-month provider runtime observations and price completion.

use ciproxy_store_interface::overflow::record::{RuntimeCharge, RuntimeTotal, StartRecord};
use ciproxy_store_interface::overflow::store::OverflowSnapshot;
use sandbox_interface::ProviderRef;
use uuid::Uuid;

use crate::cost_state;
use crate::error::{Error, Result};
use crate::maintenance::MaintenanceCapabilities;
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
            record.sandbox.is_some() && (record.ends_at.is_none() || !record.runtime_complete)
        }) {
            let Some(account) = snapshot
                .accounts
                .iter()
                .find(|account| account.account.id == record.key_id)
            else {
                self.missing_usage(session, record).await?;
                continue;
            };
            let backend = match self.maintenance_backend(workspace_id, account).await {
                Ok(backend) => backend,
                Err(_) => {
                    tracing::warn!(
                        "Boat cost credentials are unavailable; saved metadata remains readable"
                    );
                    self.missing_usage(session, record).await?;
                    continue;
                }
            };
            self.measure(session, &backend, record).await?;
        }
        Ok(())
    }

    pub(super) async fn measure(
        &self,
        session: &Session,
        backend: &MaintenanceCapabilities,
        record: &mut StartRecord,
    ) -> Result<bool> {
        let Some(sandbox) = &record.sandbox else {
            return Ok(false);
        };
        let id = ProviderRef::new(sandbox.id());
        let now = self.dependencies.clock.now();
        let total = match backend.usage(id.clone(), None, now).await {
            Ok(Some(total)) => total,
            Ok(None) | Err(_) => return self.missing_usage(session, record).await,
        };
        if record
            .runtime_total
            .as_ref()
            .is_none_or(|saved| total.until >= saved.until)
        {
            if let Some(sandbox) = &mut record.sandbox {
                sandbox.observe_creation(
                    total.since,
                    record
                        .dispatch
                        .as_ref()
                        .ok_or(Error::OwnershipUnknown)?
                        .lifetime_seconds,
                );
            }
            record.starts_at = total.since;
            record.runtime_total = Some(RuntimeTotal {
                since: total.since,
                until: total.until,
                usd: total.usd,
                running: total.running,
            });
        }
        if total.running {
            record.running_observed_at = record
                .running_observed_at
                .into_iter()
                .chain(Some(total.until))
                .max();
        }
        let mut month = spend::month_start(record.starts_at)?;
        let mut charges = Vec::new();
        let until = record.ends_at.unwrap_or(now).min(now);
        while month < until {
            let end = spend::next_month(month)?.min(until);
            let usage = match backend.usage(id.clone(), Some(month), end).await {
                Ok(Some(usage)) => usage,
                Ok(None) | Err(_) => return self.missing_usage(session, record).await,
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
        if record
            .runtime
            .iter()
            .max_by_key(|charge| charge.until)
            .is_some_and(|charge| !charge.running)
        {
            cost_state::missing_observation(record);
        }
        self.save_cost_state(session, record).await?;
        Ok(true)
    }
}
