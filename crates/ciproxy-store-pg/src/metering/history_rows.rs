//! Private Diesel rows for exact monthly and Boat cost snapshots.

use chrono::{DateTime, Utc};
use ciproxy_schema::metering::{metering_boat_costs, metering_machine_costs, metering_plan_costs};
use ciproxy_store_interface::metering::history::{BoatCost, MachineCost, PlanCost};
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

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
#[diesel(table_name = metering_boat_costs)]
pub(super) struct BoatCostRow {
    runner_id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    start_usd: Decimal,
    minute_usd: Decimal,
}

impl BoatCostRow {
    pub(super) fn into_record(self) -> BoatCost {
        BoatCost {
            runner_id: self.runner_id,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            start_usd: self.start_usd,
            minute_usd: self.minute_usd,
        }
    }
}
