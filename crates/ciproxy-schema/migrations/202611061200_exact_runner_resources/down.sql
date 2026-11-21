DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM (
            SELECT memory_bytes, disk_bytes FROM runners
            UNION ALL
            SELECT memory_bytes, disk_bytes FROM metering_job_cursors
            UNION ALL
            SELECT memory_bytes, disk_bytes FROM metering_daily_sizes
        ) AS resources
        WHERE memory_bytes % 1000000000 <> 0
           OR disk_bytes % 1000000000 <> 0
           OR memory_bytes / 1000000000 NOT BETWEEN -2147483648 AND 2147483647
           OR disk_bytes / 1000000000 NOT BETWEEN -2147483648 AND 2147483647
    ) THEN
        RAISE EXCEPTION 'exact byte resources cannot be represented as integer GB; rollback made no changes';
    END IF;
END;
$$;

ALTER TABLE runners
    ALTER COLUMN memory_bytes TYPE INTEGER USING (memory_bytes / 1000000000)::integer,
    ALTER COLUMN disk_bytes TYPE INTEGER USING (disk_bytes / 1000000000)::integer;
ALTER TABLE runners RENAME COLUMN memory_bytes TO memory_gb;
ALTER TABLE runners RENAME COLUMN disk_bytes TO disk_gb;

ALTER TABLE metering_job_cursors
    ALTER COLUMN memory_bytes TYPE INTEGER USING (memory_bytes / 1000000000)::integer,
    ALTER COLUMN disk_bytes TYPE INTEGER USING (disk_bytes / 1000000000)::integer;
ALTER TABLE metering_job_cursors RENAME COLUMN memory_bytes TO memory_gb;
ALTER TABLE metering_job_cursors RENAME COLUMN disk_bytes TO disk_gb;

ALTER TABLE metering_daily_sizes
    ALTER COLUMN memory_bytes TYPE INTEGER USING (memory_bytes / 1000000000)::integer,
    ALTER COLUMN disk_bytes TYPE INTEGER USING (disk_bytes / 1000000000)::integer;
ALTER TABLE metering_daily_sizes RENAME COLUMN memory_bytes TO memory_gb;
ALTER TABLE metering_daily_sizes RENAME COLUMN disk_bytes TO disk_gb;
