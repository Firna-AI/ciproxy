//! Trusted run-facts reuse within one workspace and organization registration.

use ciproxy_schema::github::github_jobs;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::github::RunFacts;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Jsonb};
use serde_json::Value;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

pub(super) fn lookup(
    store: &PgStore,
    workspace_id: Uuid,
    organization_id: Uuid,
    run_id: i64,
) -> Result<Option<RunFacts>> {
    let body = github_jobs::table
        .filter(github_jobs::workspace_id.eq(workspace_id))
        .filter(github_jobs::organization_id.eq(organization_id))
        .filter(sql::<BigInt>("(body->>'run_id')::bigint").eq(run_id))
        .order((github_jobs::assigned_at, github_jobs::job_id))
        .select(sql::<Jsonb>("body->'facts'"))
        .first::<Value>(&mut *store.connection()?)
        .optional()
        .store()?;
    body.map(|body| serde_json::from_value(body).internal())
        .transpose()
}
