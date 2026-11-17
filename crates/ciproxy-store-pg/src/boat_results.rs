//! Locked comparisons keep late Boat registration answers from changing newer work.

use ciproxy_schema::boat_runner_intents::boat_runner_intents;
use ciproxy_schema::github::github_jobs;
use ciproxy_schema::installation_links::organization_activation;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::boat_runtime::BoatRunnerIntent;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runtime_results::{
    BoatResultStore, BoatRunnerGuard, BoatRunnerResult,
};
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;
use crate::organization_rows::OrganizationRow;
use crate::runners::RunnerRow;

impl BoatResultStore for PgStore {
    fn boat_runner_guard(&self, workspace: Uuid, runner: Uuid) -> Result<Option<BoatRunnerGuard>> {
        transaction(&mut *self.connection()?, |connection| {
            load(connection, workspace, runner)
        })
    }

    fn apply_boat_runner_result(
        &self,
        workspace: Uuid,
        expected: &BoatRunnerGuard,
        result: BoatRunnerResult,
    ) -> Result<bool> {
        transaction(&mut *self.connection()?, |connection| {
            let id = expected.intent.runner_id;
            let Some(current) = load(connection, workspace, id)? else {
                return Ok(false);
            };
            let matches = &current == expected;
            match result {
                BoatRunnerResult::Checked => Ok(matches),
                BoatRunnerResult::Registered { target, github_id } => {
                    let mut intent = current.intent;
                    if intent.organization_id != target.organization_id
                        || intent.label_id != target.label_id
                        || intent.scale_set_id != target.scale_set_id
                        || intent.github_id.is_some_and(|id| id != github_id)
                        || intent
                            .registration_scale_set_id
                            .is_some_and(|id| id != target.github_id)
                    {
                        return Ok(false);
                    }
                    intent.github_id = Some(github_id);
                    intent.registration_scale_set_id = Some(target.github_id);
                    diesel::update(
                        boat_runner_intents::table
                            .find(id)
                            .filter(boat_runner_intents::workspace_id.eq(workspace)),
                    )
                    .set(boat_runner_intents::body.eq(serde_json::to_value(intent).internal()?))
                    .execute(connection)
                    .store()?;
                    let admitted = organization_activation::table
                        .find(expected.intent.organization_id)
                        .filter(organization_activation::workspace_id.eq(workspace))
                        .select(organization_activation::admitted)
                        .for_update()
                        .first::<bool>(connection)
                        .optional()
                        .store()?
                        .unwrap_or(false);
                    Ok(matches
                        && admitted
                        && current
                            .organization
                            .is_some_and(|organization| organization.app_state == AppState::Active))
                }
                BoatRunnerResult::Busy { github_id } => {
                    if !matches {
                        return Ok(false);
                    }
                    diesel::sql_query("UPDATE boat_runner_intents SET body=jsonb_set(body,'{github_id}',to_jsonb($3::bigint)) WHERE workspace_id=$1 AND runner_id=$2")
                        .bind::<diesel::sql_types::Uuid,_>(workspace).bind::<diesel::sql_types::Uuid,_>(id).bind::<BigInt,_>(github_id).execute(connection).store()?;
                    if current.delivery.is_some_and(|record| {
                        record.phase == StartPhase::Running && record.ends_at.is_none()
                    }) {
                        diesel::update(
                            runners::table
                                .find(id)
                                .filter(runners::workspace_id.eq(workspace)),
                        )
                        .set((runners::state.eq("busy"), runners::github_id.eq(github_id)))
                        .execute(connection)
                        .store()?;
                    }
                    Ok(true)
                }
                BoatRunnerResult::Retired => {
                    if !matches
                        || current.delivery.is_some_and(|record| {
                            record.ends_at.is_none() && record.phase != StartPhase::Deleting
                        })
                    {
                        return Ok(false);
                    }
                    diesel::delete(
                        boat_runner_intents::table
                            .find(id)
                            .filter(boat_runner_intents::workspace_id.eq(workspace)),
                    )
                    .execute(connection)
                    .store()?;
                    Ok(true)
                }
            }
        })
    }
}

fn load(
    connection: &mut PgConnection,
    workspace: Uuid,
    id: Uuid,
) -> Result<Option<BoatRunnerGuard>> {
    workspaces::table
        .find(workspace)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .store()?;
    let body = boat_runner_intents::table
        .find(id)
        .filter(boat_runner_intents::workspace_id.eq(workspace))
        .select(boat_runner_intents::body)
        .for_update()
        .first::<serde_json::Value>(connection)
        .optional()
        .store()?;
    let Some(body) = body else {
        return Ok(None);
    };
    let intent: BoatRunnerIntent = serde_json::from_value(body).internal()?;
    let organization = organizations::table
        .find(intent.organization_id)
        .filter(organizations::workspace_id.eq(workspace))
        .select(OrganizationRow::as_select())
        .for_update()
        .first::<OrganizationRow>(connection)
        .optional()
        .store()?
        .map(OrganizationRow::into_record)
        .transpose()?;
    let delivery = overflow_starts::table
        .find(intent.delivery_id)
        .filter(overflow_starts::workspace_id.eq(workspace))
        .select(overflow_starts::record_json)
        .for_update()
        .first::<String>(connection)
        .optional()
        .store()?
        .map(|json| serde_json::from_str(&json).internal())
        .transpose()?;
    let runner = runners::table
        .find(id)
        .filter(runners::workspace_id.eq(workspace))
        .select(RunnerRow::as_select())
        .for_update()
        .first::<RunnerRow>(connection)
        .optional()
        .store()?
        .map(RunnerRow::into_record)
        .transpose()?;
    let jobs = github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace))
        .filter(github_jobs::organization_id.eq(intent.organization_id))
        .filter(github_jobs::scale_set_id.eq(intent.scale_set_id))
        .order(github_jobs::job_id)
        .select(github_jobs::body)
        .for_update()
        .load::<serde_json::Value>(connection)
        .store()?
        .into_iter()
        .map(|body| serde_json::from_value(body).internal())
        .collect::<Result<Vec<GithubJob>>>()?
        .into_iter()
        .filter(|job| job.runner_github_id.is_some() && job.runner_github_id == intent.github_id)
        .collect();
    Ok(Some(BoatRunnerGuard {
        intent,
        organization,
        delivery,
        runner,
        jobs,
    }))
}
