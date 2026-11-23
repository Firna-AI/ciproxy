ALTER TABLE runners ADD COLUMN overflow_provider TEXT;
UPDATE runners SET overflow_provider = 'boat' WHERE sandbox_id IS NOT NULL;

ALTER TABLE metering_daily_overflow_costs
    DROP CONSTRAINT metering_daily_overflow_costs_pkey,
    ADD PRIMARY KEY (workspace_id, day, provider);

ALTER TABLE metering_daily_sizes
    DROP CONSTRAINT metering_daily_sizes_pkey,
    ADD COLUMN place TEXT NOT NULL DEFAULT 'machine',
    ADD PRIMARY KEY (workspace_id, day, vcpus, memory_bytes, disk_bytes, place);

UPDATE metering_daily_sizes
SET place = 'boat', machine_starts = overflow_starts, machine_minutes = overflow_minutes,
    overflow_starts = 0, overflow_minutes = 0
WHERE machine_starts = 0 AND machine_minutes = 0
    AND (overflow_starts <> 0 OR overflow_minutes <> 0);

INSERT INTO metering_daily_sizes (
    workspace_id, day, vcpus, memory_bytes, disk_bytes, place,
    machine_starts, machine_minutes, overflow_starts, overflow_minutes
)
SELECT workspace_id, day, vcpus, memory_bytes, disk_bytes, 'boat',
       overflow_starts, overflow_minutes, 0, 0
FROM metering_daily_sizes
WHERE overflow_starts <> 0 OR overflow_minutes <> 0;

ALTER TABLE metering_daily_sizes
    ALTER COLUMN place DROP DEFAULT,
    DROP COLUMN overflow_starts,
    DROP COLUMN overflow_minutes;
ALTER TABLE metering_daily_sizes RENAME COLUMN machine_starts TO starts;
ALTER TABLE metering_daily_sizes RENAME COLUMN machine_minutes TO minutes;
