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
), attempts AS (
  SELECT g.id AS attempt_id,g.workspace_id,g.organization_id,
         (g.body->>'label_id')::uuid AS label_id,
         (g.body->>'scale_set_id')::uuid AS scale_set_id,
         (g.body->>'runner_github_id')::bigint AS runner_github_id,
         g.phase,g.completed_observed_at,
         g.body || jsonb_build_object('id',g.id,
           'workflow',g.body->'workflow_ref',
           'head_branch',g.body->'facts'->'head_branch',
           'state',CASE WHEN g.phase=2 THEN 'completed'
                        WHEN g.phase=1 THEN 'running' ELSE 'queued' END,
           'runner_taken',g.phase>=1,'queued_at',g.body->'assigned_at') AS body
  FROM demands g
), aliases AS (
  SELECT j.id AS action_id,g.attempt_id
  FROM jobs j JOIN runners r ON r.id=j.runner_id
    AND r.workspace_id=j.workspace_id AND r.organization_id=j.organization_id
  JOIN attempts g ON g.workspace_id=j.workspace_id
    AND g.organization_id=j.organization_id AND g.scale_set_id=r.scale_set_id
    AND g.runner_github_id=r.github_id
    AND (g.body->>'actions_job_id')::bigint=j.github_id
  WHERE j.workspace_id=$1
), merged_actions AS (
  SELECT j.id AS action_id,j.workspace_id,j.organization_id,j.label_id,
         j.scale_set_id,j.runner_id,NULL::bigint AS runner_github_id,
         a.completed_observed_at,
         COALESCE(a.body,'{}'::jsonb) || jsonb_strip_nulls(
           to_jsonb(j) || jsonb_build_object('actions_job_id',j.github_id,
             'cache_counts',CASE WHEN j.cache_restores IS NULL THEN NULL ELSE
               jsonb_build_object('restored_entries',j.cache_restores,
                 'restored_bytes',j.cache_restored_bytes) END)) ||
         jsonb_build_object('id',j.id,
           'state',CASE WHEN a.phase=2 OR j.state='completed' THEN 'completed'
             WHEN a.phase=1 OR j.state='running' THEN 'running' ELSE j.state END,
           'runner_taken',j.started_at IS NOT NULL OR j.state<>'queued'
             OR COALESCE((a.body->>'runner_taken')::boolean,false)) AS body
  FROM jobs j LEFT JOIN LATERAL (
    SELECT g.* FROM attempts g JOIN aliases x ON x.attempt_id=g.attempt_id
    WHERE x.action_id=j.id ORDER BY g.phase DESC,
      ((g.body->>'completed_at') IS NOT NULL) DESC,
      g.completed_observed_at DESC NULLS LAST,g.attempt_id LIMIT 1
  ) a ON TRUE WHERE j.workspace_id=$1
), candidates AS (
  SELECT m.workspace_id,m.organization_id,m.label_id,m.scale_set_id,
         m.runner_id,m.runner_github_id,m.completed_observed_at,m.body
  FROM merged_actions m
  UNION ALL
  SELECT COALESCE(m.workspace_id,g.workspace_id),
         COALESCE(m.organization_id,g.organization_id),
         COALESCE(m.label_id,g.label_id),COALESCE(m.scale_set_id,g.scale_set_id),
         m.runner_id,
         CASE WHEN m.action_id IS NULL THEN g.runner_github_id ELSE m.runner_github_id END,
         CASE WHEN m.action_id IS NULL THEN g.completed_observed_at ELSE m.completed_observed_at END,
         COALESCE(m.body,g.body)
  FROM attempts g LEFT JOIN aliases x ON x.attempt_id=g.attempt_id
  LEFT JOIN merged_actions m ON m.action_id=x.action_id
), visible AS (
  SELECT DISTINCT * FROM candidates
  WHERE body->>'state'<>'completed' OR
    COALESCE((body->>'completed_at')::timestamptz,completed_observed_at,
      (body->>'started_at')::timestamptz,(body->>'queued_at')::timestamptz)>=$2
)
SELECT v.body || jsonb_build_object(
  'runner_id',r.id,
  'reported_ids',ARRAY(SELECT DISTINCT d.job_id FROM demands d
    WHERE d.organization_id=v.organization_id AND
      (d.id=(v.body->>'id')::uuid OR
        ((d.body->>'runner_github_id')::bigint=r.github_id
        AND (d.body->>'scale_set_id')::uuid=r.scale_set_id
        AND (d.body->>'actions_job_id')::bigint=(v.body->>'actions_job_id')::bigint))),
  'organization_id',v.organization_id,'organization',o.login,'label',l.name,
  'requested_size',CASE
    WHEN v.body->>'state'='queued' THEN jsonb_build_object(
      'vcpus',l.vcpus,'memory_gb',l.memory_gb,'disk_gb',l.disk_gb)
    WHEN r.machine_id IS NOT NULL THEN c.requested_size
    WHEN b.runner_id IS NOT NULL THEN jsonb_build_object(
      'vcpus',b.record_json::jsonb->'requested_vcpus',
      'memory_gb',b.record_json::jsonb->'requested_memory_gb',
      'disk_gb',b.record_json::jsonb->'requested_disk_gb')
    ELSE NULL END,
  'runner_taken',r.id IS NOT NULL AND COALESCE((v.body->>'runner_taken')::boolean,false),
  'actual_size',CASE WHEN r.id IS NULL THEN NULL ELSE jsonb_build_object(
    'vcpu',r.vcpus,'memory_bytes',r.memory_bytes,'disk_bytes',r.disk_bytes) END,
  'machine_id',r.machine_id,'machine_name',h.hostname,'first_slot',r.first_slot,
  'boat_size',r.boat_size_name) AS body
FROM visible v JOIN organizations o ON o.id=v.organization_id AND o.workspace_id=$1
JOIN labels l ON l.id=v.label_id AND l.workspace_id=$1
LEFT JOIN runners r ON r.workspace_id=$1 AND r.organization_id=v.organization_id
  AND (r.id=v.runner_id OR (v.runner_id IS NULL
    AND r.scale_set_id=v.scale_set_id AND r.github_id=v.runner_github_id))
LEFT JOIN own_runner_commands c ON c.runner_id=r.id AND c.workspace_id=$1
LEFT JOIN overflow_starts b ON b.runner_id=r.id AND b.workspace_id=$1
LEFT JOIN machines h ON h.id=r.machine_id AND h.workspace_id=$1
ORDER BY COALESCE((v.body->>'started_at')::timestamptz,
  (v.body->>'queued_at')::timestamptz) DESC NULLS LAST,v.body->>'id'
"#;
