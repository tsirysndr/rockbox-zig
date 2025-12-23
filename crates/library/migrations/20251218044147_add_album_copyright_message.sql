-- Add migration script here
ALTER TABLE album ADD COLUMN copyright_message VARCHAR(255) DEFAULT NULL;
