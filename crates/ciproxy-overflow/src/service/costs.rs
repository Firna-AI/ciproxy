//! Shared persistent updates for measured and unavailable runtime facts.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};

use crate::cost_state;
use crate::error::Result;
use crate::session::Session;

use super::OverflowService;

impl OverflowService {
    pub(super) async fn save_cost_state(
        &self,
        session: &Session,
        record: &mut StartRecord,
    ) -> Result<()> {
        let (until, clamped) = cost_state::billing_prefix(record);
        record.billed_until = until;
        if clamped {
            tracing::warn!(
                "Measured Boat billing interval exceeded its observation bounds and was clamped"
            );
        }
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await
    }

    pub(super) async fn missing_usage(
        &self,
        session: &Session,
        record: &mut StartRecord,
    ) -> Result<bool> {
        cost_state::missing_observation(record);
        self.save_cost_state(session, record).await?;
        Ok(false)
    }

    pub(super) async fn end_record(
        &self,
        session: &Session,
        record: &mut StartRecord,
        at: DateTime<Utc>,
    ) -> Result<()> {
        record.phase = StartPhase::Ended;
        record.ends_at = Some(at);
        self.save_cost_state(session, record).await
    }

    pub(super) fn proven_lifetime_end(&self, record: &StartRecord) -> Option<DateTime<Utc>> {
        record.sandbox_id.as_ref()?;
        record
            .lifetime_ends_by
            .filter(|bound| *bound <= self.dependencies.clock.now())
    }

    pub(super) async fn unavailable_lifecycle(
        &self,
        session: &Session,
        record: &mut StartRecord,
    ) -> Result<bool> {
        cost_state::observe_existence(record, self.dependencies.clock.now());
        self.missing_usage(session, record).await?;
        if let Some(bound) = self.proven_lifetime_end(record) {
            self.end_record(session, record, bound).await?;
            return Ok(true);
        }
        Ok(false)
    }
}
