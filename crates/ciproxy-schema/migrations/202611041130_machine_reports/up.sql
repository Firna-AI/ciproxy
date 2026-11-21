CREATE TABLE machine_reports (
    workspace_id UUID NOT NULL,
    machine_id UUID NOT NULL,
    session_id UUID NOT NULL,
    report JSONB NOT NULL,
    PRIMARY KEY (workspace_id, machine_id),
    FOREIGN KEY (workspace_id, machine_id)
        REFERENCES machines(workspace_id, id) ON DELETE CASCADE
);
