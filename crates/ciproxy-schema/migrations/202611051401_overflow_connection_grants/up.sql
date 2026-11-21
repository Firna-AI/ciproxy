CREATE TABLE overflow_connection_grants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL UNIQUE
        REFERENCES workspaces(id) ON DELETE CASCADE,
    key_id UUID NOT NULL
);
