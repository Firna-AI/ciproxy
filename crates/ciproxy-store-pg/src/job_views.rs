//! Tenant- and tier-scoped dashboard jobs from both trusted history sources.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::job_views::DashboardJob;
use diesel::prelude::*;
use diesel::sql_types::{Jsonb, Timestamptz};
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

#[derive(QueryableByName)]
struct Projection {
    #[diesel(sql_type = Jsonb)]
    body: serde_json::Value,
}

pub(crate) fn load(
    store: &PgStore,
    workspace_id: Uuid,
    finished_since: DateTime<Utc>,
) -> Result<Vec<DashboardJob>> {
    diesel::sql_query(QUERY)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<Timestamptz, _>(finished_since)
        .load::<Projection>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|row| serde_json::from_value(row.body).internal())
        .collect()
}

const QUERY: &str = r#"
WITH demands AS (
  SELECT id, workspace_id, organization_id, job_id, phase, body,
         completed_observed_at FROM github_job_attempts WHERE workspace_id=$1
  UNION ALL
  SELECT job_id, workspace_id, organization_id, job_id, phase, body,
         completed_observed_at FROM github_jobs g WHERE workspace_id=$1
    AND NOT EXISTS (SELECT 1 FROM github_job_attempts a
      WHERE a.workspace_id=g.workspace_id AND a.organization_id=g.organization_id
      AND a.job_id=g.job_id)
), metadata AS (
  SELECT j.workspace_id, j.organization_id, j.label_id, j.scale_set_id,
         j.runner_id, NULL::bigint AS runner_github_id,
         to_jsonb(j) || jsonb_build_object('actions_job_id',j.github_id,
           'run_id',a.body->'run_id','head_branch',a.body->'facts'->'head_branch',
           'state',CASE WHEN a.phase=2 OR j.state='completed' THEN 'completed'
             WHEN a.phase=1 OR j.state='running' THEN 'running' ELSE j.state END,
           'started_at',COALESCE(to_jsonb(j.started_at),a.body->'started_at'),
           'completed_at',COALESCE(to_jsonb(j.completed_at),a.body->'completed_at'),
           'cache_counts', CASE WHEN j.cache_restores IS NULL THEN NULL ELSE
             jsonb_build_object('restored_entries',j.cache_restores,
               'restored_bytes',j.cache_restored_bytes) END) AS body
  FROM jobs j LEFT JOIN LATERAL (
    SELECT g.body,g.phase,g.completed_observed_at FROM demands g JOIN runners r ON r.id=j.runner_id
      AND r.workspace_id=j.workspace_id AND r.organization_id=j.organization_id
    WHERE g.workspace_id=j.workspace_id AND g.organization_id=j.organization_id
      AND (g.body->>'actions_job_id')::bigint=j.github_id
      AND (g.body->>'runner_github_id')::bigint=r.github_id
      AND (g.body->>'scale_set_id')::uuid=r.scale_set_id ORDER BY g.id LIMIT 1
  ) a ON TRUE WHERE j.workspace_id=$1
    AND (j.started_at IS NOT NULL OR a.body IS NULL)
    AND ((j.state<>'completed' AND COALESCE(a.phase,-1)<>2) OR
      COALESCE(j.completed_at,(a.body->>'completed_at')::timestamptz,
        a.completed_observed_at,j.started_at,j.queued_at)>=$2)
  UNION ALL
  SELECT g.workspace_id, g.organization_id, (g.body->>'label_id')::uuid,
         (g.body->>'scale_set_id')::uuid, NULL::uuid,
         (g.body->>'runner_github_id')::bigint,
         g.body || jsonb_build_object('id',g.id,
           'workflow',g.body->'workflow_ref',
           'head_branch',g.body->'facts'->'head_branch',
           'state',CASE WHEN g.phase=2 THEN 'completed'
                        WHEN g.phase=1 THEN 'running' ELSE 'queued' END,
           'queued_at',g.body->'assigned_at')
  FROM demands g WHERE
    (g.phase<>2 OR COALESCE((g.body->>'completed_at')::timestamptz,
      g.completed_observed_at,(g.body->>'assigned_at')::timestamptz)>=$2)
    AND NOT EXISTS (SELECT 1 FROM jobs j JOIN runners r ON r.id=j.runner_id
      AND r.workspace_id=j.workspace_id AND r.organization_id=j.organization_id
      WHERE j.workspace_id=g.workspace_id AND j.organization_id=g.organization_id
        AND j.github_id=(g.body->>'actions_job_id')::bigint
        AND j.started_at IS NOT NULL
        AND r.scale_set_id=(g.body->>'scale_set_id')::uuid
        AND r.github_id=(g.body->>'runner_github_id')::bigint)
)
SELECT m.body || jsonb_build_object(
  'organization_id',m.organization_id,'organization',o.login,'label',l.name,
  'requested_vcpus',l.vcpus,'requested_memory_gb',l.memory_gb,
  'requested_disk_gb',l.disk_gb,
  'actual_size',CASE WHEN r.id IS NULL THEN NULL ELSE jsonb_build_object(
    'vcpu',r.vcpus,'memory_bytes',r.memory_bytes,'disk_bytes',r.disk_bytes) END,
  'machine_id',r.machine_id,'machine_name',h.hostname,'first_slot',r.first_slot,
  'boat_size',r.boat_size_name) AS body
FROM metadata m JOIN organizations o ON o.id=m.organization_id AND o.workspace_id=$1
JOIN labels l ON l.id=m.label_id AND l.workspace_id=$1
LEFT JOIN runners r ON r.workspace_id=$1 AND r.organization_id=m.organization_id
  AND (r.id=m.runner_id OR (m.runner_id IS NULL
    AND r.scale_set_id=m.scale_set_id AND r.github_id=m.runner_github_id))
LEFT JOIN machines h ON h.id=r.machine_id AND h.workspace_id=$1
ORDER BY COALESCE((m.body->>'started_at')::timestamptz,
  (m.body->>'queued_at')::timestamptz) DESC NULLS LAST, m.body->>'id'
"#;
