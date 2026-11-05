ALTER TABLE workspaces ADD COLUMN metering_started_at TIMESTAMPTZ;
ALTER TABLE workspaces ADD COLUMN metering_retained_from DATE;
ALTER TABLE machines ADD COLUMN registered_at TIMESTAMPTZ;
ALTER TABLE jobs ADD UNIQUE (workspace_id, id);
ALTER TABLE runners ADD UNIQUE (workspace_id, id);

CREATE TABLE metering_machine_costs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    machine_id UUID NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    monthly_usd NUMERIC NOT NULL,
    FOREIGN KEY (workspace_id, machine_id)
        REFERENCES machines(workspace_id, id) ON DELETE CASCADE
);
CREATE TABLE metering_plan_costs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    monthly_usd NUMERIC NOT NULL
);
CREATE TABLE metering_boat_costs (
    runner_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    start_usd NUMERIC NOT NULL,
    minute_usd NUMERIC NOT NULL,
    FOREIGN KEY (workspace_id, runner_id)
        REFERENCES runners(workspace_id, id) ON DELETE CASCADE
);
CREATE TABLE metering_periods (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (workspace_id, starts_at)
);
CREATE TABLE metering_daily_sizes (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    day DATE NOT NULL,
    vcpus INTEGER NOT NULL,
    memory_gb INTEGER NOT NULL,
    disk_gb INTEGER NOT NULL,
    machine_starts BIGINT NOT NULL,
    machine_minutes NUMERIC NOT NULL,
    boat_starts BIGINT NOT NULL,
    boat_minutes NUMERIC NOT NULL,
    PRIMARY KEY (workspace_id, day, vcpus, memory_gb, disk_gb)
);
CREATE TABLE metering_daily_boat_costs (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    day DATE NOT NULL,
    usd NUMERIC NOT NULL,
    PRIMARY KEY (workspace_id, day)
);
CREATE TABLE metering_start_totals (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    taken_at TIMESTAMPTZ NOT NULL,
    starts BIGINT NOT NULL,
    PRIMARY KEY (workspace_id, taken_at)
);
CREATE TABLE metering_job_cursors (
    job_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    observed_at TIMESTAMPTZ NOT NULL,
    vcpus INTEGER NOT NULL,
    memory_gb INTEGER NOT NULL,
    disk_gb INTEGER NOT NULL,
    place TEXT NOT NULL,
    FOREIGN KEY (workspace_id, job_id)
        REFERENCES jobs(workspace_id, id) ON DELETE CASCADE
);
CREATE TABLE metering_boat_cursors (
    runner_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    start_usd NUMERIC NOT NULL,
    minute_usd NUMERIC NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (workspace_id, runner_id)
        REFERENCES runners(workspace_id, id) ON DELETE CASCADE
);
CREATE INDEX metering_machine_costs_workspace ON metering_machine_costs (workspace_id);
CREATE INDEX metering_plan_costs_workspace ON metering_plan_costs (workspace_id);
CREATE INDEX metering_boat_costs_workspace ON metering_boat_costs (workspace_id);
CREATE INDEX metering_job_cursors_workspace ON metering_job_cursors (workspace_id);
CREATE INDEX metering_boat_cursors_workspace ON metering_boat_cursors (workspace_id);
