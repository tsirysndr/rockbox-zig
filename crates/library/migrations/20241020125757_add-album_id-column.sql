-- Add migration script here
ALTER TABLE favourites ADD COLUMN album_id VARCHAR(255) DEFAULT NULL;
