//! Locked saved authority for one pending own-machine remote operation.

use ciproxy_schema::github::github_jobs;
use ciproxy_schema::installation_links::organization_activation;
use ciproxy_schema::machines::machines;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::own_runners::own_runner_commands;
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::own_runners::OwnRunner;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::runtime_results::OwnRunnerGuard;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::gateway_contacts;
use crate::machine_reads;
use crate::organization_rows::OrganizationRow;
use crate::own_runners::{CommandRow, into_record};
use crate::runners::RunnerRow;

pub(crate) fn command(
    connection: &mut PgConnection,
    workspace: Uuid,
    command: Uuid,
) -> Result<Option<OwnRunner>> {
    own_runner_commands::table
        .inner_join(
            runners::table.on(runners::id
                .eq(own_runner_commands::runner_id)
                .and(runners::workspace_id.eq(own_runner_commands::workspace_id))),
        )
        .filter(own_runner_commands::workspace_id.eq(workspace))
        .filter(own_runner_commands::command_id.eq(command))
        .select((CommandRow::as_select(), RunnerRow::as_select()))
        .first::<(CommandRow, RunnerRow)>(connection)
        .optional()
        .store()?
        .map(into_record)
        .transpose()
}

pub(crate) fn load(
    connection: &mut PgConnection,
    workspace: Uuid,
    id: Uuid,
) -> Result<Option<OwnRunnerGuard>> {
    workspaces::table
        .find(workspace)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .store()?;
    let owner = runners::table
        .find(id)
        .filter(runners::workspace_id.eq(workspace))
        .select((runners::organization_id, runners::machine_id))
        .first::<(Uuid, Option<Uuid>)>(connection)
        .optional()
        .store()?;
    let Some((organization, machine_id)) = owner else {
        return Ok(None);
    };
    let organization = organizations::table
        .find(organization)
        .filter(organizations::workspace_id.eq(workspace))
        .select(OrganizationRow::as_select())
        .for_update()
        .first::<OrganizationRow>(connection)
        .optional()
        .store()?
        .map(OrganizationRow::into_record)
        .transpose()?;
    if let Some(machine) = machine_id {
        machines::table
            .find(machine)
            .filter(machines::workspace_id.eq(workspace))
            .select(machines::id)
            .for_update()
            .load::<Uuid>(connection)
            .store()?;
    }
    let runner = runners::table
        .find(id)
        .filter(runners::workspace_id.eq(workspace))
        .select(RunnerRow::as_select())
        .for_update()
        .first::<RunnerRow>(connection)
        .optional()
        .store()?;
    let command = own_runner_commands::table
        .find(id)
        .filter(own_runner_commands::workspace_id.eq(workspace))
        .select(CommandRow::as_select())
        .for_update()
        .first::<CommandRow>(connection)
        .optional()
        .store()?;
    let (Some(runner), Some(command)) = (runner, command) else {
        return Ok(None);
    };
    let runner = into_record((command, runner))?;
    let admitted = organization_activation::table
        .find(runner.runner.organization_id)
        .filter(organization_activation::workspace_id.eq(workspace))
        .select(organization_activation::admitted)
        .for_update()
        .first::<bool>(connection)
        .optional()
        .store()?
        .unwrap_or(false);
    let jobs = github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace))
        .filter(github_jobs::organization_id.eq(runner.runner.organization_id))
        .filter(github_jobs::scale_set_id.eq(runner.runner.scale_set_id))
        .order(github_jobs::job_id)
        .select(github_jobs::body)
        .for_update()
        .load::<serde_json::Value>(connection)
        .store()?
        .into_iter()
        .map(|body| serde_json::from_value(body).internal())
        .collect::<Result<Vec<GithubJob>>>()?
        .into_iter()
        .filter(|job| {
            job.runner_github_id.is_some() && job.runner_github_id == runner.runner.github_id
        })
        .collect();
    let (machine, session_id) = match runner.runner.place {
        RunnerPlace::Machine { machine_id, .. } => (
            machine_reads::load_snapshots(connection, workspace)?
                .into_iter()
                .find(|machine| machine.machine.id == machine_id),
            gateway_contacts::session(connection, workspace, machine_id)?,
        ),
        _ => (None, None),
    };
    Ok(Some(OwnRunnerGuard {
        runner,
        organization,
        machine,
        session_id,
        admitted,
        jobs,
    }))
}
