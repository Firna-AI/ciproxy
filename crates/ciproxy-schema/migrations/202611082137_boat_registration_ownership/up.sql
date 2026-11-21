ALTER TABLE boat_runner_intents
    ADD COLUMN installation_id BIGINT,
    ADD COLUMN delete_after TIMESTAMPTZ;

UPDATE boat_runner_intents i
SET installation_id = o.installation_id
FROM organizations o
WHERE o.id = i.organization_id AND o.workspace_id = i.workspace_id;

CREATE VIEW boat_registration_ownership AS
SELECT
    i.workspace_id,
    i.runner_id,
    (
        o.id IS NULL OR o.removed_at IS NOT NULL OR
        (o.app_state = 'removed' AND t.completed IS TRUE
            AND t.installation_id = o.installation_id)
    ) AS github_access_ended,
    CASE
        WHEN i.installation_id IS NULL THEN NULL
        WHEN o.id IS NULL THEN i.delete_after
        WHEN o.app_state = 'removed' AND o.removed_at IS NOT NULL
            THEN o.delete_after
        WHEN o.app_state = 'removed' AND t.completed IS TRUE
            AND t.installation_id = o.installation_id
            THEN '-infinity'::timestamptz
        ELSE NULL
    END AS forget_after,
    (
        NOT EXISTS (
            SELECT 1 FROM overflow_starts s
            WHERE (s.delivery_id = i.delivery_id OR s.runner_id = i.runner_id)
                AND (
                    s.workspace_id IS DISTINCT FROM i.workspace_id
                    OR s.delivery_id IS DISTINCT FROM i.delivery_id
                    OR s.runner_id IS DISTINCT FROM i.runner_id
                    OR s.record_json::jsonb ->> 'organization_id'
                        IS DISTINCT FROM i.organization_id::text
                    OR s.ends_at IS NULL
                )
        )
        AND NOT EXISTS (
            SELECT 1 FROM runners r WHERE r.id = i.runner_id
                AND (
                    r.workspace_id IS DISTINCT FROM i.workspace_id
                    OR r.organization_id IS DISTINCT FROM i.organization_id
                    OR r.state <> 'stopped'
                )
        )
    ) AS terminal
FROM boat_runner_intents i
LEFT JOIN organizations o
    ON o.id = i.organization_id AND o.workspace_id = i.workspace_id
LEFT JOIN dashboard_takeouts t
    ON t.organization_id = o.id AND t.workspace_id = i.workspace_id;
