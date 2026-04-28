ALTER TABLE track ADD COLUMN is_remote INTEGER NOT NULL DEFAULT 0;
UPDATE track SET is_remote = 1 WHERE path LIKE 'http://%' OR path LIKE 'https://%';
