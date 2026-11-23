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

CREATE FUNCTION runner_has_job_usage_history(runner runners) RETURNS BOOLEAN LANGUAGE sql STABLE
RETURN EXISTS (
    SELECT 1 FROM jobs AS job
    WHERE job.workspace_id = runner.workspace_id AND job.runner_id = runner.id
) OR EXISTS (
    SELECT 1 FROM github_job_attempts AS attempt
    WHERE attempt.workspace_id = runner.workspace_id
        AND attempt.organization_id = runner.organization_id
        AND (attempt.body->>'scale_set_id')::uuid = runner.scale_set_id
        AND (attempt.body->>'runner_github_id')::bigint = runner.github_id
) OR EXISTS (
    SELECT 1 FROM github_jobs AS demand
    WHERE demand.workspace_id = runner.workspace_id
        AND demand.organization_id = runner.organization_id
        AND demand.scale_set_id = runner.scale_set_id
        AND (demand.body->>'runner_github_id')::bigint = runner.github_id
);

CREATE FUNCTION remove_actions_job_usage() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM job_usage_samples AS sample USING runners AS runner
    WHERE sample.workspace_id = OLD.workspace_id AND sample.runner_id = OLD.runner_id
        AND runner.workspace_id = sample.workspace_id AND runner.id = sample.runner_id
        AND NOT runner_has_job_usage_history(runner);
    RETURN OLD;
END;
$$;

CREATE TRIGGER actions_job_usage_cleanup AFTER DELETE ON jobs
    FOR EACH ROW WHEN (OLD.runner_id IS NOT NULL)
    EXECUTE FUNCTION remove_actions_job_usage();

CREATE FUNCTION remove_scale_job_usage() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM job_usage_samples AS sample USING runners AS runner
    WHERE runner.workspace_id = OLD.workspace_id AND runner.organization_id = OLD.organization_id
        AND runner.scale_set_id = (OLD.body->>'scale_set_id')::uuid
        AND runner.github_id = (OLD.body->>'runner_github_id')::bigint
        AND sample.workspace_id = runner.workspace_id AND sample.runner_id = runner.id
        AND NOT runner_has_job_usage_history(runner);
    RETURN OLD;
END;
$$;

CREATE TRIGGER scale_attempt_job_usage_cleanup AFTER DELETE ON github_job_attempts
    FOR EACH ROW WHEN (OLD.body->>'runner_github_id' IS NOT NULL)
    EXECUTE FUNCTION remove_scale_job_usage();

CREATE TRIGGER scale_demand_job_usage_cleanup AFTER DELETE ON github_jobs
    FOR EACH ROW WHEN (OLD.body->>'runner_github_id' IS NOT NULL)
    EXECUTE FUNCTION remove_scale_job_usage();
