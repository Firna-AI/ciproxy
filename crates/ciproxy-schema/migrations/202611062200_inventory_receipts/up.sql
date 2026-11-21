ALTER TABLE machine_reports
    ADD COLUMN received_at TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
ALTER TABLE machine_reports ALTER COLUMN received_at DROP DEFAULT;

ALTER TABLE own_runner_commands ADD COLUMN confirmed_at TIMESTAMPTZ;
UPDATE own_runner_commands SET confirmed_at = CURRENT_TIMESTAMP WHERE acknowledged;
