-- Add migration script here
ALTER TABLE album ADD COLUMN label VARCHAR(255) DEFAULT NULL;
