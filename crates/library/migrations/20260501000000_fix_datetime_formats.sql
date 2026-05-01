-- Repair rows where created_at or updated_at is an empty string
UPDATE track SET created_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE created_at IS NULL OR created_at = '';
UPDATE track SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE updated_at IS NULL OR updated_at = '';

-- Convert SQLite CURRENT_TIMESTAMP format (YYYY-MM-DD HH:MM:SS, no T separator) to RFC3339
-- SQLx expects RFC3339 when decoding DateTime<Utc> from text
UPDATE track SET created_at = strftime('%Y-%m-%dT%H:%M:%SZ', created_at) WHERE created_at NOT LIKE '%T%';
UPDATE track SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) WHERE updated_at NOT LIKE '%T%';
