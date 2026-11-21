CREATE TABLE boat_runner_intents (
    runner_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    delivery_id UUID NOT NULL UNIQUE,
    organization_id UUID NOT NULL,
    label_id UUID NOT NULL,
    scale_set_id UUID NOT NULL,
    body JSONB NOT NULL
);
CREATE INDEX boat_runner_intents_workspace ON boat_runner_intents(workspace_id);
