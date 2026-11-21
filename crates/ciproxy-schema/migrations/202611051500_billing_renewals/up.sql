CREATE TABLE billing_renewals (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    subscription_id TEXT NOT NULL,
    invoice_id TEXT NOT NULL,
    outcome_json TEXT NOT NULL,
    PRIMARY KEY (workspace_id, invoice_id)
);

CREATE INDEX billing_renewals_subscription
    ON billing_renewals (workspace_id, subscription_id);
