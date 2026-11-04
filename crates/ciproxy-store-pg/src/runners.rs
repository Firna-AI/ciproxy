//! Runner insertion, lifecycle updates, and private placement mappings.

use ciproxy_schema::runners::runners;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState, RunnerStore};
use ciproxy_store_interface::vm_size::VmSize;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{InternalCause, QueryResultExt, require_record};

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = runners)]
struct RunnerRow {
    id: Uuid,
    workspace_id: Uuid,
    organization_id: Uuid,
    label_id: Uuid,
    scale_set_id: Uuid,
    github_id: Option<i64>,
    state: String,
    vcpus: i32,
    memory_gb: i32,
    disk_gb: i32,
    machine_id: Option<Uuid>,
    first_slot: Option<i32>,
    boat_key_id: Option<Uuid>,
    sandbox_id: Option<String>,
    boat_size_name: Option<String>,
}

impl RunnerRow {
    fn from_record(workspace_id: Uuid, runner: &Runner) -> Self {
        let (machine_id, first_slot, boat_key_id, sandbox_id, boat_size_name) = match &runner.place
        {
            RunnerPlace::Machine {
                machine_id,
                first_slot,
            } => (Some(*machine_id), Some(*first_slot), None, None, None),
            RunnerPlace::Boat {
                key_id,
                sandbox_id,
                size_name,
            } => (
                None,
                None,
                *key_id,
                Some(sandbox_id.clone()),
                Some(size_name.clone()),
            ),
        };
        let state: &str = runner.state.into();
        Self {
            id: runner.id,
            workspace_id,
            organization_id: runner.organization_id,
            label_id: runner.label_id,
            scale_set_id: runner.scale_set_id,
            github_id: runner.github_id,
            state: state.to_owned(),
            vcpus: runner.size.vcpus,
            memory_gb: runner.size.memory_gb,
            disk_gb: runner.size.disk_gb,
            machine_id,
            first_slot,
            boat_key_id,
            sandbox_id,
            boat_size_name,
        }
    }

    fn into_record(self) -> Result<Runner> {
        let place = match (
            self.machine_id,
            self.first_slot,
            self.sandbox_id,
            self.boat_size_name,
        ) {
            (Some(machine_id), Some(first_slot), None, None) => RunnerPlace::Machine {
                machine_id,
                first_slot,
            },
            (None, None, Some(sandbox_id), Some(size_name)) => RunnerPlace::Boat {
                key_id: self.boat_key_id,
                sandbox_id,
                size_name,
            },
            _ => return Err(Error::internal(InternalCause::InvalidRow)),
        };
        Ok(Runner {
            id: self.id,
            organization_id: self.organization_id,
            label_id: self.label_id,
            scale_set_id: self.scale_set_id,
            github_id: self.github_id,
            state: self.state.parse().internal()?,
            size: VmSize {
                vcpus: self.vcpus,
                memory_gb: self.memory_gb,
                disk_gb: self.disk_gb,
            },
            place,
        })
    }
}

impl RunnerStore for PgStore {
    fn create_runner(&self, workspace_id: Uuid, runner: &Runner) -> Result<()> {
        diesel::insert_into(runners::table)
            .values(RunnerRow::from_record(workspace_id, runner))
            .execute(&mut *self.connection()?)
            .store()?;
        Ok(())
    }

    fn runners(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<Runner>> {
        runners::table
            .filter(runners::workspace_id.eq(workspace_id))
            .filter(runners::organization_id.eq(organization_id))
            .order(runners::id)
            .select(RunnerRow::as_select())
            .load(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(RunnerRow::into_record)
            .collect()
    }

    fn update_runner(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        id: Uuid,
        state: RunnerState,
        github_id: Option<i64>,
    ) -> Result<()> {
        let state: &str = state.into();
        let count = diesel::update(
            runners::table
                .filter(runners::workspace_id.eq(workspace_id))
                .filter(runners::organization_id.eq(organization_id))
                .find(id),
        )
        .set((runners::state.eq(state), runners::github_id.eq(github_id)))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }
}
