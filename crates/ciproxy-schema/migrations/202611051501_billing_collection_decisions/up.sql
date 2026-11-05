UPDATE billing_renewals
SET outcome_json = (
    (outcome_json::jsonb - 'automatic_stop' - 'collection_changed_at')
    || jsonb_build_object(
        'collection', jsonb_build_object(
            'changed_at', outcome_json::jsonb -> 'collection_changed_at',
            'decisions', CASE
                WHEN jsonb_typeof(outcome_json::jsonb -> 'automatic_stop') = 'object'
                THEN jsonb_build_array(jsonb_build_object(
                    'stop', outcome_json::jsonb -> 'automatic_stop',
                    'state', 'eligible'
                ))
                ELSE '[]'::jsonb
            END
        )
    )
)::text;
