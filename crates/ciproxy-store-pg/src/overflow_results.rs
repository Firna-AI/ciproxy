//! Locked comparisons keep late registration answers from changing newer work.

use ciproxy_schema::github::github_jobs;
use ciproxy_schema::installation_links::organization_activation;
use ciproxy_schema::organizations::organizations;
use ciproxy_schema::overflow::overflow_starts;
use ciproxy_schema::overflow_runner_intents::overflow_runner_intents;
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::overflow_delivery::OverflowDeliveryGuard;
use ciproxy_store_interface::overflow_runtime::OverflowRunnerIntent;
use ciproxy_store_interface::runtime_results::{
    OverflowResultStore, OverflowRunnerGuard, OverflowRunnerResult, OverflowRunnerWrite,
};
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::transaction;
use crate::organization_rows::OrganizationRow;
use crate::runners::RunnerRow;

impl OverflowResultStore for PgStore {
    fn overflow_runner_guard(
        &self,
        workspace: Uuid,
        runner: Uuid,
    ) -> Result<Option<OverflowRunnerGuard>> {
        transaction(&mut *self.connection()?, |connection| {
            load(connection, workspace, runner)
        })
    }

    fn apply_overflow_runner_result(
        &self,
        workspace: Uuid,
        expected: &OverflowRunnerGuard,
        result: OverflowRunnerResult,
    ) -> Result<OverflowRunnerWrite> {
        transaction(&mut *self.connection()?, |connection| {
            let id = expected.intent.runner_id;
            let Some(current) = load(connection, workspace, id)? else {
                return Ok(OverflowRunnerWrite::Rejected);
            };
            let matches = &current == expected;
            let accepted = match result {
                OverflowRunnerResult::Checked => matches,
                OverflowRunnerResult::Registered { target, github_id } => {
                    let mut intent = current.intent;
                    if intent.organization_id != target.organization_id
                        || intent.label_id != target.label_id
                        || intent.scale_set_id != target.scale_set_id
                        || intent.github_id.is_some_and(|id| id != github_id)
                        || intent
                            .registration_scale_set_id
                            .is_some_and(|id| id != target.github_id)
                    {
                        return Ok(OverflowRunnerWrite::Rejected);
                    }
                    intent.github_id = Some(github_id);
                    intent.registration_scale_set_id = Some(target.github_id);
                    diesel::update(
                        overflow_runner_intents::table
                            .find(id)
                            .filter(overflow_runner_intents::workspace_id.eq(workspace)),
                    )
                    .set(overflow_runner_intents::body.eq(serde_json::to_value(intent).internal()?))
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
                    matches
                        && admitted
                        && current
                            .organization
                            .is_some_and(|organization| organization.app_state == AppState::Active)
                }
                OverflowRunnerResult::Busy { github_id } => {
                    if !matches {
                        return Ok(OverflowRunnerWrite::Rejected);
                    }
                    diesel::sql_query("UPDATE overflow_runner_intents SET body=jsonb_set(body,'{github_id}',to_jsonb($3::bigint)) WHERE workspace_id=$1 AND runner_id=$2")
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
                    true
                }
                OverflowRunnerResult::Retired => {
                    if !matches
                        || current.delivery.is_some_and(|record| {
                            record.ends_at.is_none() && record.phase != StartPhase::Deleting
                        })
                    {
                        return Ok(OverflowRunnerWrite::Rejected);
                    }
                    diesel::delete(
                        overflow_runner_intents::table
                            .find(id)
                            .filter(overflow_runner_intents::workspace_id.eq(workspace)),
                    )
                    .execute(connection)
                    .store()?;
                    return Ok(OverflowRunnerWrite::Retired);
                }
            };
            if !accepted {
                return Ok(OverflowRunnerWrite::Rejected);
            }
            Ok(load(connection, workspace, id)?
                .map(|guard| OverflowRunnerWrite::Applied(Box::new(guard)))
                .unwrap_or(OverflowRunnerWrite::Rejected))
        })
    }
}

/// Keep delivery foreign-key checks compatible with the workspace fence.
fn load(
    connection: &mut PgConnection,
    workspace: Uuid,
    id: Uuid,
) -> Result<Option<OverflowRunnerGuard>> {
    workspaces::table
        .find(workspace)
        .select(workspaces::id)
        .for_no_key_update()
        .first::<Uuid>(connection)
        .store()?;
    let body = overflow_runner_intents::table
        .find(id)
        .filter(overflow_runner_intents::workspace_id.eq(workspace))
        .select(overflow_runner_intents::body)
        .for_update()
        .first::<serde_json::Value>(connection)
        .optional()
        .store()?;
    let Some(body) = body else {
        return Ok(None);
    };
    let intent: OverflowRunnerIntent = serde_json::from_value(body).internal()?;
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
        .map(|json| serde_json::from_str::<StartRecord>(&json).internal())
        .transpose()?
        .map(OverflowDeliveryGuard::from);
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
    Ok(Some(OverflowRunnerGuard {
        intent,
        organization,
        delivery,
        runner,
        jobs,
    }))
}
