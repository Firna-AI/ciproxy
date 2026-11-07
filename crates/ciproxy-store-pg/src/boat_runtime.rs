//! Durable Boat registration identities and their scoped runner mappings.

use chrono::{DateTime, Utc};
use ciproxy_schema::boat_runner_intents::boat_runner_intents;
use ciproxy_store_interface::boat_runtime::{BoatRunnerIntent, BoatRuntimeStore};
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::overflow::store::OverflowSnapshot;
use ciproxy_store_interface::scale_sets::RegistrationTarget;
use diesel::prelude::*;
use diesel::sql_types::{self, BigInt, Jsonb};
use uuid::Uuid;

use crate::boat_registration_ownership;
use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;

impl BoatRuntimeStore for PgStore {
    fn save_boat_registration(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        target: RegistrationTarget,
        github_id: i64,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            let count = diesel::sql_query(
                "UPDATE boat_runner_intents SET body=jsonb_set(jsonb_set(body,'{github_id}',to_jsonb($3::bigint)),
                 '{registration_scale_set_id}',to_jsonb($4::bigint))
                 WHERE workspace_id=$1 AND runner_id=$2 AND organization_id=$5 AND label_id=$6 AND scale_set_id=$7
                 AND (body->>'registration_scale_set_id' IS NULL OR (body->>'registration_scale_set_id')::bigint=$4)",
            ).bind::<sql_types::Uuid,_>(workspace_id).bind::<sql_types::Uuid,_>(runner_id)
                .bind::<BigInt,_>(github_id).bind::<BigInt,_>(target.github_id)
                .bind::<sql_types::Uuid,_>(target.organization_id).bind::<sql_types::Uuid,_>(target.label_id)
                .bind::<sql_types::Uuid,_>(target.scale_set_id).execute(connection).store()?;
            if count == 0 {
                return Err(Error::NotFound);
            }
            diesel::sql_query("UPDATE runners SET github_id=$3,registration_scale_set_id=$4 WHERE workspace_id=$1 AND id=$2 AND organization_id=$5 AND label_id=$6 AND scale_set_id=$7 AND sandbox_id IS NOT NULL AND (registration_scale_set_id IS NULL OR registration_scale_set_id=$4)")
                .bind::<sql_types::Uuid,_>(workspace_id).bind::<sql_types::Uuid,_>(runner_id)
                .bind::<BigInt,_>(github_id).bind::<BigInt,_>(target.github_id)
                .bind::<sql_types::Uuid,_>(target.organization_id).bind::<sql_types::Uuid,_>(target.label_id)
                .bind::<sql_types::Uuid,_>(target.scale_set_id).execute(connection).store()?;
            Ok(())
        })
    }
    fn boat_runtime_snapshot(&self, workspace_id: Uuid) -> Result<OverflowSnapshot> {
        crate::overflow::rows::snapshot(&mut *self.connection()?, workspace_id)
    }
    fn boat_runner_intents(&self, workspace_id: Uuid) -> Result<Vec<BoatRunnerIntent>> {
        load(&mut *self.connection()?, workspace_id)
    }

    fn boat_registration_access_ended(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<bool> {
        boat_registration_ownership::access_ended(&mut *self.connection()?, workspace_id, runner_id)
            .store()
    }

    fn purge_terminal_boat_runner_intents(
        &self,
        workspace_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<usize> {
        self.connection()?
            .transaction(|connection| {
                diesel::sql_query("SELECT id FROM workspaces WHERE id=$1 FOR UPDATE")
                    .bind::<sql_types::Uuid, _>(workspace_id)
                    .execute(connection)?;
                boat_registration_ownership::purge(connection, workspace_id, at)
            })
            .store()
    }

    fn create_boat_runner_intent(
        &self,
        workspace_id: Uuid,
        intent: &BoatRunnerIntent,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            diesel::sql_query("SELECT id FROM workspaces WHERE id=$1 FOR UPDATE")
                .bind::<sql_types::Uuid, _>(workspace_id)
                .execute(connection)
                .store()?;
            let saved = diesel::sql_query(
            "INSERT INTO boat_runner_intents
             (runner_id,workspace_id,delivery_id,organization_id,label_id,scale_set_id,body,installation_id)
             SELECT $1,$2,$3,$4,$5,$6,$7,o.installation_id FROM workspaces w
             JOIN settings s ON s.workspace_id=w.id
             JOIN organizations o ON o.workspace_id=w.id AND o.id=$4
             JOIN organization_activation a ON a.workspace_id=w.id AND a.organization_id=o.id
             JOIN labels l ON l.workspace_id=w.id AND l.id=$5
             JOIN scale_sets b ON b.workspace_id=w.id AND b.organization_id=o.id AND b.label_id=l.id AND b.id=$6
             WHERE w.id=$2 AND o.app_state='active' AND a.admitted
             AND s.overflow_enabled AND l.deleted_at IS NULL AND l.allow_boat AND b.remote_removed_at IS NULL
             AND l.vcpus=($7::jsonb->'requested_size'->>'vcpu')::bigint
             AND l.memory_gb::numeric*1000000000=($7::jsonb->'requested_size'->>'memory_bytes')::numeric
             AND l.disk_gb::numeric*1000000000=($7::jsonb->'requested_size'->>'disk_bytes')::numeric
             AND EXISTS (SELECT 1 FROM boat_accounts k WHERE k.workspace_id=w.id AND k.retired_at IS NULL AND k.rejected_at IS NULL)",
        )
        .bind::<sql_types::Uuid,_>(intent.runner_id).bind::<sql_types::Uuid,_>(workspace_id)
        .bind::<sql_types::Uuid,_>(intent.delivery_id).bind::<sql_types::Uuid,_>(intent.organization_id)
        .bind::<sql_types::Uuid,_>(intent.label_id).bind::<sql_types::Uuid,_>(intent.scale_set_id)
        .bind::<Jsonb,_>(serde_json::to_value(intent).internal()?)
        .execute(connection).store()?;
            if saved == 0 {
                return Err(Error::Conflict);
            }
            Ok(())
        })
    }

    fn register_boat_runner(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        github_id: i64,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            let updated = diesel::sql_query(
                "UPDATE boat_runner_intents SET body=jsonb_set(body,'{github_id}',to_jsonb($3::bigint)) WHERE workspace_id=$1 AND runner_id=$2",
            ).bind::<sql_types::Uuid,_>(workspace_id).bind::<sql_types::Uuid,_>(runner_id)
                .bind::<BigInt,_>(github_id).execute(connection).store()?;
            if updated == 0 {
                return Err(Error::NotFound);
            }
            diesel::sql_query("UPDATE runners SET github_id=$3 WHERE workspace_id=$1 AND id=$2 AND sandbox_id IS NOT NULL")
                .bind::<sql_types::Uuid,_>(workspace_id).bind::<sql_types::Uuid,_>(runner_id)
                .bind::<BigInt,_>(github_id).execute(connection).store()?;
            Ok(())
        })
    }

    fn remove_boat_runner_intent(&self, workspace_id: Uuid, runner_id: Uuid) -> Result<()> {
        diesel::delete(
            boat_runner_intents::table
                .filter(boat_runner_intents::workspace_id.eq(workspace_id))
                .filter(boat_runner_intents::runner_id.eq(runner_id)),
        )
        .execute(&mut *self.connection()?)
        .store()?;
        Ok(())
    }
}

pub(crate) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<Vec<BoatRunnerIntent>> {
    boat_runner_intents::table
        .filter(boat_runner_intents::workspace_id.eq(workspace_id))
        .order(boat_runner_intents::runner_id)
        .select(boat_runner_intents::body)
        .load::<serde_json::Value>(connection)
        .store()?
        .into_iter()
        .map(|body| serde_json::from_value(body).internal())
        .collect()
}
