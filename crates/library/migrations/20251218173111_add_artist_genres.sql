-- Add migration script here
CREATE TABLE IF NOT EXISTS artist_genres (
    id VARCHAR(255) PRIMARY KEY,
    artist_id VARCHAR(255) NOT NULL,
    genre_id VARCHAR(255) NOT NULL,
    UNIQUE (artist_id, genre_id)
);

ALTER TABLE artist ADD COLUMN genres VARCHAR(255) DEFAULT NULL;

ALTER TABLE genre ADD CONSTRAINT unique_genre_name UNIQUE (name);
