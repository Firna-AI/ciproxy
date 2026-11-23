//! Private Diesel rows for exact monthly and overflow cost snapshots.

use chrono::{DateTime, Utc};
use ciproxy_schema::metering::{
    metering_machine_costs, metering_overflow_costs, metering_plan_costs,
};
use ciproxy_store_interface::metering::history::{MachineCost, OverflowCost, PlanCost};
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::overflow_provider::DbOverflowProvider;

#[derive(Queryable, Selectable)]
#[diesel(table_name = metering_machine_costs)]
pub(super) struct MachineCostRow {
    id: Uuid,
    machine_id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    monthly_usd: Decimal,
}

impl MachineCostRow {
    pub(super) fn into_record(self) -> MachineCost {
        MachineCost {
            id: self.id,
            machine_id: self.machine_id,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            monthly_usd: self.monthly_usd,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = metering_plan_costs)]
pub(super) struct PlanCostRow {
    id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    monthly_usd: Decimal,
}

impl PlanCostRow {
    pub(super) fn into_record(self) -> PlanCost {
        PlanCost {
            id: self.id,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            monthly_usd: self.monthly_usd,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = metering_overflow_costs)]
pub(super) struct OverflowCostRow {
    runner_id: Uuid,
    provider: DbOverflowProvider,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    start_usd: Decimal,
    minute_usd: Decimal,
}

impl OverflowCostRow {
    pub(super) fn into_record(self) -> OverflowCost {
        OverflowCost {
            runner_id: self.runner_id,
            provider: self.provider.0,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            start_usd: self.start_usd,
            minute_usd: self.minute_usd,
        }
    }
}
