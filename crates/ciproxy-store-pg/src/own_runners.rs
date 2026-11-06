//! Atomic own-machine runner reservations and credential-free command queries.

use chrono::{DateTime, Utc};
use ciproxy_schema::own_runners::own_runner_commands;
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::error::ResultExt;
use ciproxy_store_interface::own_runners::{OwnRunner, OwnRunnerStore};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::transaction;
use crate::runners::RunnerRow;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = own_runner_commands)]
struct CommandRow {
    runner_id: Uuid,
    workspace_id: Uuid,
    label_name: String,
    command_id: Uuid,
    issued_at: DateTime<Utc>,
    acknowledged: bool,
    slot_indices: Vec<i64>,
    requested_size: serde_json::Value,
}

impl OwnRunnerStore for PgStore {
    fn save_own_size(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        requested_size: ciproxy_store_interface::vm_size::VmSize,
        allocation: ciproxy_protocol::resources::VmSize,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            diesel::update(
                own_runner_commands::table
                    .find(runner_id)
                    .filter(own_runner_commands::workspace_id.eq(workspace_id)),
            )
            .set(
                own_runner_commands::requested_size
                    .eq(serde_json::to_value(requested_size).internal()?),
            )
            .execute(connection)
            .store()?;
            diesel::update(
                runners::table
                    .find(runner_id)
                    .filter(runners::workspace_id.eq(workspace_id)),
            )
            .set((
                runners::vcpus.eq(i32::try_from(allocation.vcpu).internal()?),
                runners::memory_bytes.eq(i64::try_from(allocation.memory_bytes).internal()?),
                runners::disk_bytes.eq(i64::try_from(allocation.disk_bytes).internal()?),
            ))
            .execute(connection)
            .store()?;
            Ok(())
        })
    }
    fn save_own_slots(&self, workspace_id: Uuid, runner_id: Uuid, slots: &[u64]) -> Result<()> {
        let indices = slots
            .iter()
            .map(|slot| i64::try_from(*slot))
            .collect::<std::result::Result<Vec<_>, _>>()
            .internal()?;
        transaction(&mut *self.connection()?, |connection| {
            diesel::update(
                own_runner_commands::table
                    .find(runner_id)
                    .filter(own_runner_commands::workspace_id.eq(workspace_id)),
            )
            .set(own_runner_commands::slot_indices.eq(&indices))
            .execute(connection)
            .store()?;
            if let Some(first) = slots.first() {
                diesel::update(
                    runners::table
                        .find(runner_id)
                        .filter(runners::workspace_id.eq(workspace_id)),
                )
                .set(runners::first_slot.eq(i32::try_from(*first).internal()?))
                .execute(connection)
                .store()?;
            }
            Ok(())
        })
    }
    fn own_routing_snapshot(
        &self,
        workspace_id: Uuid,
    ) -> Result<ciproxy_store_interface::own_runners::RoutingSnapshot> {
        crate::routing_snapshot::snapshot(&mut *self.connection()?, workspace_id)
    }
    fn try_lock_own_runners(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<Box<dyn ciproxy_store_interface::own_runners::OwnRunnerLease>>> {
        crate::own_lease::acquire(self, workspace_id)
    }
    fn routing_workspaces(&self) -> Result<Vec<Uuid>> {
        workspaces::table
            .select(workspaces::id)
            .order(workspaces::id)
            .load(&mut *self.connection()?)
            .store()
    }

    fn own_runners(&self, workspace_id: Uuid) -> Result<Vec<OwnRunner>> {
        load(&mut *self.connection()?, workspace_id)
    }

    fn create_own_runner(&self, workspace_id: Uuid, runner: &OwnRunner) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            crate::own_reservation::require_authority(connection, workspace_id, runner)?;
            diesel::insert_into(runners::table)
                .values(RunnerRow::from_record(workspace_id, &runner.runner)?)
                .execute(connection)
                .store()?;
            diesel::insert_into(own_runner_commands::table)
                .values(CommandRow {
                    runner_id: runner.runner.id,
                    workspace_id,
                    label_name: runner.label_name.clone(),
                    command_id: runner.command_id,
                    issued_at: runner.issued_at,
                    acknowledged: false,
                    slot_indices: runner
                        .slot_indices
                        .iter()
                        .map(|slot| i64::try_from(*slot))
                        .collect::<std::result::Result<Vec<_>, _>>()
                        .internal()?,
                    requested_size: serde_json::to_value(runner.requested_size).internal()?,
                })
                .execute(connection)
                .store()?;
            Ok(())
        })
    }

    fn retire_own_runner(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
        command_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        transaction(&mut *self.connection()?, |connection| {
            let count = diesel::update(
                runners::table
                    .filter(runners::workspace_id.eq(workspace_id))
                    .find(runner_id),
            )
            .set(runners::state.eq("stopping"))
            .execute(connection)
            .store()?;
            require_record(count)?;
            diesel::update(
                own_runner_commands::table
                    .filter(own_runner_commands::workspace_id.eq(workspace_id))
                    .find(runner_id),
            )
            .set((
                own_runner_commands::command_id.eq(command_id),
                own_runner_commands::issued_at.eq(at),
                own_runner_commands::acknowledged.eq(false),
            ))
            .execute(connection)
            .store()?;
            Ok(())
        })
    }

    fn acknowledge_own_runner(&self, workspace_id: Uuid, command_id: Uuid) -> Result<()> {
        diesel::update(
            own_runner_commands::table
                .filter(own_runner_commands::workspace_id.eq(workspace_id))
                .filter(own_runner_commands::command_id.eq(command_id)),
        )
        .set(own_runner_commands::acknowledged.eq(true))
        .execute(&mut *self.connection()?)
        .store()?;
        Ok(())
    }
}

pub(crate) fn load(connection: &mut PgConnection, workspace_id: Uuid) -> Result<Vec<OwnRunner>> {
    own_runner_commands::table
        .inner_join(
            runners::table.on(runners::id
                .eq(own_runner_commands::runner_id)
                .and(runners::workspace_id.eq(own_runner_commands::workspace_id))),
        )
        .filter(own_runner_commands::workspace_id.eq(workspace_id))
        .order(own_runner_commands::runner_id)
        .select((CommandRow::as_select(), RunnerRow::as_select()))
        .load::<(CommandRow, RunnerRow)>(connection)
        .store()?
        .into_iter()
        .map(|(command, runner)| {
            Ok(OwnRunner {
                requested_size: serde_json::from_value(command.requested_size).internal()?,
                slot_indices: command
                    .slot_indices
                    .into_iter()
                    .map(u64::try_from)
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .internal()?,
                runner: runner.into_record()?,
                label_name: command.label_name,
                command_id: command.command_id,
                issued_at: command.issued_at,
                acknowledged: command.acknowledged,
            })
        })
        .collect()
}
