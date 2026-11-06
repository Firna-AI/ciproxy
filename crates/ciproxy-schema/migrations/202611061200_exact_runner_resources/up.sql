ALTER TABLE runners RENAME COLUMN memory_gb TO memory_bytes;
ALTER TABLE runners RENAME COLUMN disk_gb TO disk_bytes;
ALTER TABLE runners
    ALTER COLUMN memory_bytes TYPE BIGINT USING memory_bytes::bigint * 1000000000,
    ALTER COLUMN disk_bytes TYPE BIGINT USING disk_bytes::bigint * 1000000000;

ALTER TABLE metering_job_cursors RENAME COLUMN memory_gb TO memory_bytes;
ALTER TABLE metering_job_cursors RENAME COLUMN disk_gb TO disk_bytes;
ALTER TABLE metering_job_cursors
    ALTER COLUMN memory_bytes TYPE BIGINT USING memory_bytes::bigint * 1000000000,
    ALTER COLUMN disk_bytes TYPE BIGINT USING disk_bytes::bigint * 1000000000;

ALTER TABLE metering_daily_sizes RENAME COLUMN memory_gb TO memory_bytes;
ALTER TABLE metering_daily_sizes RENAME COLUMN disk_gb TO disk_bytes;
ALTER TABLE metering_daily_sizes
    ALTER COLUMN memory_bytes TYPE BIGINT USING memory_bytes::bigint * 1000000000,
    ALTER COLUMN disk_bytes TYPE BIGINT USING disk_bytes::bigint * 1000000000;
