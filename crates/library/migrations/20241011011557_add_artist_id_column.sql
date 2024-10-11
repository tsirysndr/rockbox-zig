-- Add migration script here
ALTER TABLE album ADD COLUMN artist_id VARCHAR(255) DEFAULT NULL;
