ALTER TABLE metering_daily_sizes RENAME COLUMN starts TO machine_starts;
ALTER TABLE metering_daily_sizes RENAME COLUMN minutes TO machine_minutes;
ALTER TABLE metering_daily_sizes
    ADD COLUMN overflow_starts BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN overflow_minutes NUMERIC NOT NULL DEFAULT 0;

WITH overflow AS (
    DELETE FROM metering_daily_sizes WHERE place <> 'machine'
    RETURNING workspace_id, day, vcpus, memory_bytes, disk_bytes, machine_starts, machine_minutes
)
INSERT INTO metering_daily_sizes (
    workspace_id, day, vcpus, memory_bytes, disk_bytes, place,
    machine_starts, machine_minutes, overflow_starts, overflow_minutes
)
SELECT workspace_id, day, vcpus, memory_bytes, disk_bytes, 'machine',
       0, 0, SUM(machine_starts), SUM(machine_minutes)
FROM overflow
GROUP BY workspace_id, day, vcpus, memory_bytes, disk_bytes
ON CONFLICT (workspace_id, day, vcpus, memory_bytes, disk_bytes, place)
DO UPDATE SET overflow_starts = EXCLUDED.overflow_starts,
              overflow_minutes = EXCLUDED.overflow_minutes;

ALTER TABLE metering_daily_sizes
    DROP CONSTRAINT metering_daily_sizes_pkey,
    DROP COLUMN place,
    ALTER COLUMN overflow_starts DROP DEFAULT,
    ALTER COLUMN overflow_minutes DROP DEFAULT,
    ADD PRIMARY KEY (workspace_id, day, vcpus, memory_bytes, disk_bytes);

ALTER TABLE metering_daily_overflow_costs
    DROP CONSTRAINT metering_daily_overflow_costs_pkey,
    ADD PRIMARY KEY (workspace_id, day);

ALTER TABLE runners DROP COLUMN overflow_provider;
