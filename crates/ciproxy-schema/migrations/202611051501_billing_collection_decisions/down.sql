UPDATE billing_renewals
SET outcome_json = (
    (outcome_json::jsonb - 'collection')
    || jsonb_build_object(
        'collection_changed_at', outcome_json::jsonb -> 'collection' -> 'changed_at',
        'automatic_stop', (
            SELECT decision -> 'stop'
            FROM jsonb_array_elements(
                outcome_json::jsonb -> 'collection' -> 'decisions'
            ) AS decision
            WHERE decision ->> 'state' = 'eligible'
            LIMIT 1
        )
    )
)::text;
