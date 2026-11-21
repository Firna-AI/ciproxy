CREATE TABLE github_job_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    job_id UUID NOT NULL,
    assignment_order TEXT NOT NULL,
    phase SMALLINT NOT NULL,
    completed_observed_at TIMESTAMPTZ,
    body JSONB NOT NULL,
    UNIQUE (workspace_id, organization_id, job_id, assignment_order),
    FOREIGN KEY (organization_id, job_id)
        REFERENCES github_jobs(organization_id, job_id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, organization_id)
        REFERENCES organizations(workspace_id, id) ON DELETE CASCADE
);

INSERT INTO github_job_attempts (workspace_id,organization_id,job_id,assignment_order,phase,body,completed_observed_at)
    SELECT workspace_id,organization_id,job_id,assignment_order,phase,body,completed_observed_at
    FROM github_jobs WHERE assignment_order <> '';

ALTER TABLE metering_job_cursors
    DROP CONSTRAINT metering_job_cursors_workspace_id_job_id_fkey,
    DROP CONSTRAINT metering_job_cursors_pkey,
    ADD COLUMN source TEXT NOT NULL DEFAULT 'actions',
    ADD PRIMARY KEY (workspace_id, organization_id, source, job_id);

ALTER TABLE metering_job_cursors ADD COLUMN facts JSONB;
UPDATE metering_job_cursors SET facts = jsonb_build_object(
    'source', 'actions', 'id', job_id, 'organization_id', organization_id,
    'started_at', started_at,
    'completion', CASE WHEN completed_at IS NULL THEN jsonb_build_object('state','running')
        ELSE jsonb_build_object('state','finished','at',completed_at) END,
    'size', jsonb_build_object('vcpu',vcpus,'memory_bytes',memory_bytes,'disk_bytes',disk_bytes),
    'place', place
);
ALTER TABLE metering_job_cursors ALTER COLUMN facts SET NOT NULL,
    DROP COLUMN started_at, DROP COLUMN completed_at, DROP COLUMN vcpus,
    DROP COLUMN memory_bytes, DROP COLUMN disk_bytes, DROP COLUMN place,
    ALTER COLUMN observed_at TYPE TEXT USING
      to_char(observed_at AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US') || '000Z';
ALTER TABLE metering_start_totals ALTER COLUMN taken_at TYPE TEXT USING
    to_char(taken_at AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US') || '000Z';

CREATE FUNCTION remove_actions_metering_cursor() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM metering_job_cursors WHERE source = 'actions'
        AND workspace_id = OLD.workspace_id AND organization_id = OLD.organization_id
        AND job_id = OLD.id;
    RETURN OLD;
END;
$$;
CREATE TRIGGER actions_metering_cursor_cleanup AFTER DELETE ON jobs
    FOR EACH ROW EXECUTE FUNCTION remove_actions_metering_cursor();

CREATE FUNCTION remove_scale_metering_cursor() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM metering_job_cursors WHERE source = 'scale_set'
        AND workspace_id = OLD.workspace_id AND organization_id = OLD.organization_id
        AND job_id = OLD.id;
    RETURN OLD;
END;
$$;
CREATE TRIGGER scale_metering_cursor_cleanup AFTER DELETE ON github_job_attempts
    FOR EACH ROW EXECUTE FUNCTION remove_scale_metering_cursor();
