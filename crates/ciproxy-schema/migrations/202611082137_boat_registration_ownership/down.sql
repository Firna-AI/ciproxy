DROP VIEW boat_registration_ownership;
ALTER TABLE boat_runner_intents
    DROP COLUMN delete_after,
    DROP COLUMN installation_id;
