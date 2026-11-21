DROP TRIGGER scale_metering_cursor_cleanup ON github_job_attempts;
DROP FUNCTION remove_scale_metering_cursor();
DROP TRIGGER actions_metering_cursor_cleanup ON jobs;
DROP FUNCTION remove_actions_metering_cursor();
DELETE FROM metering_job_cursors WHERE source = 'scale_set';
ALTER TABLE metering_job_cursors ADD COLUMN started_at TIMESTAMPTZ,
    ADD COLUMN completed_at TIMESTAMPTZ, ADD COLUMN vcpus INTEGER,
    ADD COLUMN memory_bytes BIGINT, ADD COLUMN disk_bytes BIGINT, ADD COLUMN place TEXT;
UPDATE metering_job_cursors SET started_at=(facts->>'started_at')::timestamptz,
    completed_at=(facts->'completion'->>'at')::timestamptz,
    vcpus=(facts->'size'->>'vcpu')::integer,
    memory_bytes=(facts->'size'->>'memory_bytes')::bigint,
    disk_bytes=(facts->'size'->>'disk_bytes')::bigint, place=facts->>'place';
ALTER TABLE metering_job_cursors DROP COLUMN facts,
    ALTER COLUMN started_at SET NOT NULL, ALTER COLUMN vcpus SET NOT NULL,
    ALTER COLUMN memory_bytes SET NOT NULL, ALTER COLUMN disk_bytes SET NOT NULL,
    ALTER COLUMN place SET NOT NULL,
    ALTER COLUMN observed_at TYPE TIMESTAMPTZ USING observed_at::timestamptz;
ALTER TABLE metering_start_totals ALTER COLUMN taken_at TYPE TIMESTAMPTZ USING taken_at::timestamptz;
ALTER TABLE metering_job_cursors DROP CONSTRAINT metering_job_cursors_pkey,
    DROP COLUMN source, ADD PRIMARY KEY (job_id),
    ADD FOREIGN KEY (workspace_id, job_id)
        REFERENCES jobs(workspace_id, id) ON DELETE CASCADE;
DROP TABLE github_job_attempts;
