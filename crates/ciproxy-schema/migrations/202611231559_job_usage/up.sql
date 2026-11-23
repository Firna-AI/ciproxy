CREATE TABLE job_usage_samples (
    workspace_id UUID NOT NULL,
    runner_id UUID NOT NULL,
    platform TEXT NOT NULL CHECK (platform IN ('linux_x64', 'macos_arm64')),
    first_point_at_ms BIGINT NOT NULL,
    point_seconds BIGINT NOT NULL CHECK (point_seconds > 0),
    cpu_milli_vcpu BIGINT[] NOT NULL,
    peak_memory_mib BIGINT[] NOT NULL,
    oom_kills BIGINT[],
    is_final BOOLEAN NOT NULL,
    received_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (workspace_id, runner_id),
    FOREIGN KEY (workspace_id, runner_id)
        REFERENCES runners(workspace_id, id) ON DELETE CASCADE,
    CHECK (cardinality(cpu_milli_vcpu) = cardinality(peak_memory_mib)),
    CHECK ((platform = 'linux_x64') = (oom_kills IS NOT NULL)),
    CHECK (oom_kills IS NULL OR cardinality(cpu_milli_vcpu) = cardinality(oom_kills))
);

CREATE TABLE job_usage_steps (
    workspace_id UUID NOT NULL,
    runner_id UUID NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL,
    started_at_ms BIGINT,
    ended_at_ms BIGINT,
    skipped BOOLEAN NOT NULL,
    PRIMARY KEY (workspace_id, runner_id, position),
    FOREIGN KEY (workspace_id, runner_id)
        REFERENCES job_usage_samples(workspace_id, runner_id) ON DELETE CASCADE,
    CHECK (NOT skipped OR (started_at_ms IS NULL AND ended_at_ms IS NULL))
);

CREATE INDEX job_usage_samples_arrival ON job_usage_samples (workspace_id, received_at);
CREATE INDEX runners_scale_set_github_id ON runners (workspace_id, organization_id, scale_set_id, github_id)
    WHERE github_id IS NOT NULL;

CREATE INDEX jobs_job_usage_runner ON jobs (workspace_id, runner_id)
    WHERE runner_id IS NOT NULL;
CREATE INDEX github_job_attempts_job_usage_runner ON github_job_attempts
    (workspace_id, organization_id, ((body->>'scale_set_id')::uuid), ((body->>'runner_github_id')::bigint))
    WHERE (body->>'runner_github_id')::bigint IS NOT NULL;
CREATE INDEX github_jobs_job_usage_runner ON github_jobs
    (workspace_id, organization_id, scale_set_id, ((body->>'runner_github_id')::bigint))
    WHERE (body->>'runner_github_id')::bigint IS NOT NULL;
CREATE INDEX github_job_attempts_parent_job ON github_job_attempts (organization_id, job_id);

CREATE VIEW job_usage_runners_without_history AS
SELECT runner.workspace_id, runner.id, runner.organization_id, runner.scale_set_id, runner.github_id
FROM runners AS runner
WHERE NOT EXISTS (
    SELECT 1 FROM jobs AS job
    WHERE job.workspace_id = runner.workspace_id AND job.runner_id = runner.id
) AND NOT EXISTS (
    SELECT 1 FROM github_job_attempts AS attempt
    WHERE attempt.workspace_id = runner.workspace_id
        AND attempt.organization_id = runner.organization_id
        AND (attempt.body->>'scale_set_id')::uuid = runner.scale_set_id
        AND (attempt.body->>'runner_github_id')::bigint = runner.github_id
) AND NOT EXISTS (
    SELECT 1 FROM github_jobs AS demand
    WHERE demand.workspace_id = runner.workspace_id
        AND demand.organization_id = runner.organization_id
        AND demand.scale_set_id = runner.scale_set_id
        AND (demand.body->>'runner_github_id')::bigint = runner.github_id
);

CREATE FUNCTION remove_actions_job_usage() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM job_usage_samples AS sample
    USING (
        SELECT DISTINCT workspace_id, runner_id FROM deleted_jobs WHERE runner_id IS NOT NULL
    ) AS affected, job_usage_runners_without_history AS runner
    WHERE sample.workspace_id = affected.workspace_id AND sample.runner_id = affected.runner_id
        AND runner.workspace_id = sample.workspace_id AND runner.id = sample.runner_id;
    RETURN NULL;
END;
$$;

CREATE TRIGGER actions_job_usage_cleanup AFTER DELETE ON jobs
    REFERENCING OLD TABLE AS deleted_jobs FOR EACH STATEMENT
    EXECUTE FUNCTION remove_actions_job_usage();

CREATE FUNCTION remove_scale_attempt_job_usage() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM job_usage_samples AS sample
    USING (
        SELECT DISTINCT workspace_id, organization_id, (body->>'scale_set_id')::uuid AS scale_set_id,
            (body->>'runner_github_id')::bigint AS github_id
        FROM deleted_attempts WHERE body->>'runner_github_id' IS NOT NULL
    ) AS affected, job_usage_runners_without_history AS runner
    WHERE runner.workspace_id = affected.workspace_id AND runner.organization_id = affected.organization_id
        AND runner.scale_set_id = affected.scale_set_id AND runner.github_id = affected.github_id
        AND sample.workspace_id = runner.workspace_id AND sample.runner_id = runner.id;
    RETURN NULL;
END;
$$;

CREATE TRIGGER scale_attempt_job_usage_cleanup AFTER DELETE ON github_job_attempts
    REFERENCING OLD TABLE AS deleted_attempts FOR EACH STATEMENT
    EXECUTE FUNCTION remove_scale_attempt_job_usage();

CREATE FUNCTION remove_scale_demand_job_usage() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM job_usage_samples AS sample
    USING (
        SELECT DISTINCT workspace_id, organization_id, scale_set_id,
            (body->>'runner_github_id')::bigint AS github_id
        FROM deleted_demand WHERE body->>'runner_github_id' IS NOT NULL
    ) AS affected, job_usage_runners_without_history AS runner
    WHERE runner.workspace_id = affected.workspace_id AND runner.organization_id = affected.organization_id
        AND runner.scale_set_id = affected.scale_set_id AND runner.github_id = affected.github_id
        AND sample.workspace_id = runner.workspace_id AND sample.runner_id = runner.id;
    RETURN NULL;
END;
$$;

CREATE TRIGGER scale_demand_job_usage_cleanup AFTER DELETE ON github_jobs
    REFERENCING OLD TABLE AS deleted_demand FOR EACH STATEMENT
    EXECUTE FUNCTION remove_scale_demand_job_usage();
