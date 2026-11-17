//! Atomic conditional writes for owned remote runner tasks.

use ciproxy_schema::own_runners::own_runner_commands;
use ciproxy_schema::runners::runners;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runtime_results::{
    OwnRunnerGuard, OwnRunnerResult, RuntimeResultStore,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;
use crate::own_result_guard;

impl RuntimeResultStore for PgStore {
    fn own_command(&self, workspace_id: Uuid, command_id: Uuid) -> Result<Option<OwnRunner>> {
        own_result_guard::command(&mut *self.connection()?, workspace_id, command_id)
    }

    fn own_runner_guard(
        &self,
        workspace_id: Uuid,
        runner_id: Uuid,
    ) -> Result<Option<OwnRunnerGuard>> {
        transaction(&mut *self.connection()?, |connection| {
            own_result_guard::load(connection, workspace_id, runner_id)
        })
    }

    fn apply_own_runner_result(
        &self,
        workspace_id: Uuid,
        expected: &OwnRunnerGuard,
        result: OwnRunnerResult,
    ) -> Result<bool> {
        transaction(&mut *self.connection()?, |connection| {
            let id = expected.runner.runner.id;
            let Some(current) = own_result_guard::load(connection, workspace_id, id)? else {
                return Ok(false);
            };
            let matches = same_authority(&current, expected);
            if let OwnRunnerResult::Registered { target, github_id } = result {
                let changed = diesel::update(
                    runners::table
                        .find(id)
                        .filter(runners::workspace_id.eq(workspace_id))
                        .filter(runners::organization_id.eq(target.organization_id))
                        .filter(runners::label_id.eq(target.label_id))
                        .filter(runners::scale_set_id.eq(target.scale_set_id))
                        .filter(
                            runners::github_id
                                .is_null()
                                .or(runners::github_id.eq(github_id)),
                        )
                        .filter(
                            runners::registration_scale_set_id
                                .is_null()
                                .or(runners::registration_scale_set_id.eq(target.github_id)),
                        ),
                )
                .set((
                    runners::github_id.eq(github_id),
                    runners::registration_scale_set_id.eq(target.github_id),
                ))
                .execute(connection)
                .store()?;
                return Ok(matches
                    && changed == 1
                    && current.admitted
                    && current
                        .organization
                        .is_some_and(|organization| organization.app_state == AppState::Active)
                    && current
                        .machine
                        .is_some_and(|machine| !machine.machine.draining));
            }
            if !matches {
                return Ok(false);
            }
            match result {
                OwnRunnerResult::Retired {
                    command_id,
                    at,
                    github_id,
                } => {
                    diesel::update(
                        runners::table
                            .find(id)
                            .filter(runners::workspace_id.eq(workspace_id)),
                    )
                    .set((
                        runners::state.eq("stopping"),
                        runners::github_id.eq(github_id),
                    ))
                    .execute(connection)
                    .store()?;
                    diesel::update(
                        own_runner_commands::table
                            .find(id)
                            .filter(own_runner_commands::workspace_id.eq(workspace_id)),
                    )
                    .set((
                        own_runner_commands::command_id.eq(command_id),
                        own_runner_commands::issued_at.eq(at),
                        own_runner_commands::acknowledged.eq(false),
                    ))
                    .execute(connection)
                    .store()?;
                }
                OwnRunnerResult::Busy { github_id, at } => {
                    diesel::update(
                        runners::table
                            .find(id)
                            .filter(runners::workspace_id.eq(workspace_id)),
                    )
                    .set((runners::state.eq("busy"), runners::github_id.eq(github_id)))
                    .execute(connection)
                    .store()?;
                    diesel::update(
                        own_runner_commands::table
                            .find(id)
                            .filter(own_runner_commands::workspace_id.eq(workspace_id)),
                    )
                    .set(
                        own_runner_commands::confirmed_at
                            .eq(expected.runner.confirmed_at.or(Some(at))),
                    )
                    .execute(connection)
                    .store()?;
                }
                OwnRunnerResult::Stopped { github_id } => {
                    diesel::update(
                        runners::table
                            .find(id)
                            .filter(runners::workspace_id.eq(workspace_id)),
                    )
                    .set((
                        runners::state.eq("stopped"),
                        runners::github_id.eq(github_id),
                    ))
                    .execute(connection)
                    .store()?;
                }
                OwnRunnerResult::Prepared => {
                    diesel::update(
                        own_runner_commands::table
                            .find(id)
                            .filter(own_runner_commands::workspace_id.eq(workspace_id)),
                    )
                    .set(own_runner_commands::command_id.eq(expected.runner.command_id))
                    .execute(connection)
                    .store()?;
                }
                OwnRunnerResult::Registered { .. } => return Ok(false),
            }
            Ok(true)
        })
    }
}

fn same_authority(current: &OwnRunnerGuard, expected: &OwnRunnerGuard) -> bool {
    let runner_id = expected.runner.runner.id.to_string();
    let machine_authority = |guard: &OwnRunnerGuard| {
        guard.machine.as_ref().map(|machine| {
            (
                machine.machine.draining,
                machine
                    .report
                    .as_ref()
                    .and_then(|report| {
                        report
                            .report
                            .runners
                            .iter()
                            .find(|runner| runner.runner_id.0 == runner_id)
                    })
                    .cloned(),
            )
        })
    };
    current.runner == expected.runner
        && current.organization == expected.organization
        && current.session_id == expected.session_id
        && current.admitted == expected.admitted
        && current.jobs == expected.jobs
        && machine_authority(current) == machine_authority(expected)
}
