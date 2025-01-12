-- Add migration script here
ALTER TABLE playlist_tracks ADD COLUMN position INT NOT NULL DEFAULT 0;
