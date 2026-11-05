//! Immutable runner placements and compatible metering cost snapshots.

use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::QueryResultExt;

pub(super) fn save(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
) -> Result<()> {
    let state = match record.phase {
        StartPhase::Ended => "stopped",
        StartPhase::Deleting => "stopping",
        _ => "starting",
    };
    diesel::sql_query(
        "INSERT INTO runners (id, workspace_id, organization_id, label_id, scale_set_id,
            state, vcpus, memory_gb, disk_gb, boat_key_id, sandbox_id, boat_size_name)
         SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
         WHERE EXISTS (SELECT 1 FROM organizations WHERE id = $3 AND workspace_id = $2)
         AND EXISTS (SELECT 1 FROM scale_sets WHERE id = $5 AND workspace_id = $2)
         ON CONFLICT (id) DO UPDATE SET state =
            CASE WHEN EXCLUDED.state = 'stopped' THEN 'stopped'
            WHEN runners.state IN ('busy', 'stopped', 'ready') THEN runners.state ELSE EXCLUDED.state END
         WHERE runners.workspace_id = $2 AND runners.organization_id = $3
            AND runners.sandbox_id = $11",
    )
    .bind::<diesel::sql_types::Uuid, _>(record.runner_id)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Uuid, _>(record.organization_id)
    .bind::<diesel::sql_types::Uuid, _>(record.label_id)
    .bind::<diesel::sql_types::Uuid, _>(record.scale_set_id)
    .bind::<Text, _>(state)
    .bind::<Integer, _>(i32::try_from(record.size.vcpus).internal()?)
    .bind::<Integer, _>(i32::try_from(record.size.memory_bytes / 1_000_000_000).internal()?)
    .bind::<Integer, _>(i32::try_from(record.size.disk_bytes / 1_000_000_000).internal()?)
    .bind::<diesel::sql_types::Uuid, _>(record.key_id)
    .bind::<Nullable<Text>, _>(&record.sandbox_id)
    .bind::<Text, _>(&record.size.name)
    .execute(connection).store()?;
    if let Some(start_usd) = record.start_usd.filter(|_| record.runtime_complete) {
        diesel::sql_query(
            "INSERT INTO metering_boat_costs
                (runner_id, workspace_id, starts_at, ends_at, start_usd, minute_usd)
             SELECT $1, $2, $3, $4, $5, $6 WHERE EXISTS
                (SELECT 1 FROM runners WHERE id = $1 AND workspace_id = $2)
             ON CONFLICT (runner_id) DO UPDATE SET starts_at = EXCLUDED.starts_at,
                ends_at = EXCLUDED.ends_at, start_usd = EXCLUDED.start_usd,
                minute_usd = EXCLUDED.minute_usd",
        )
        .bind::<diesel::sql_types::Uuid, _>(record.runner_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<Timestamptz, _>(record.starts_at)
        .bind::<Nullable<Timestamptz>, _>(record.ends_at)
        .bind::<diesel::sql_types::Numeric, _>(start_usd)
        .bind::<diesel::sql_types::Numeric, _>(record.size.hourly_usd / Decimal::from(60))
        .execute(connection)
        .store()?;
    }
    Ok(())
}
