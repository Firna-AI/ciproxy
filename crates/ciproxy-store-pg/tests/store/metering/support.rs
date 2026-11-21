//! Literal pricing and timestamp helpers for metering integration tests.

use chrono::{DateTime, Utc};
use ciproxy_metering::report::catalog::{Alternative, PriceList, SizePrice};
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::store::support::Database;

pub(super) fn historical_workspace_start(database: &Database, workspace: Uuid, at: DateTime<Utc>) {
    let mut connection = PgConnection::establish(&database.url).unwrap();
    diesel::sql_query("UPDATE workspaces SET metering_started_at=$1 WHERE id=$2")
        .bind::<sql_types::Timestamptz, _>(at)
        .bind::<sql_types::Uuid, _>(workspace)
        .execute(&mut connection)
        .unwrap();
}

pub(super) fn historical_registration(
    database: &Database,
    workspace: Uuid,
    machine: Uuid,
    at: DateTime<Utc>,
) {
    let mut connection = PgConnection::establish(&database.url).unwrap();
    diesel::sql_query("UPDATE machines SET registered_at=$1 WHERE workspace_id=$2 AND id=$3")
        .bind::<sql_types::Timestamptz, _>(at)
        .bind::<sql_types::Uuid, _>(workspace)
        .bind::<sql_types::Uuid, _>(machine)
        .execute(&mut connection)
        .unwrap();
}

pub(super) fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}
pub(super) fn usd(value: &str) -> Decimal {
    value.parse().unwrap()
}
pub(super) fn free() -> Selection {
    Selection {
        tier: Tier::Free,
        add_ons: AddOns::default(),
    }
}

pub(super) fn prices() -> [PriceList; 2] {
    [
        PriceList {
            alternative: Alternative::Blacksmith,
            dated: time("2026-11-04T00:00:00Z").date_naive(),
            sizes: vec![SizePrice {
                vcpus: 2,
                memory_gb: 8,
                minute_usd: usd("1"),
            }],
        },
        PriceList {
            alternative: Alternative::Github,
            dated: time("2026-11-04T00:00:00Z").date_naive(),
            sizes: vec![SizePrice {
                vcpus: 2,
                memory_gb: 8,
                minute_usd: usd("2"),
            }],
        },
    ]
}
