-- Collapse duplicate genre rows by name and enforce UNIQUE(name).
--
-- Background: the previous migration (20251218173111_add_artist_genres.sql) tried
-- to add UNIQUE(name) via `ALTER TABLE genre ADD CONSTRAINT ...`, which SQLite
-- doesn't support. The error was swallowed by the warn!() fallback, so the
-- constraint was never enforced and update_metadata's `INSERT OR IGNORE` (which
-- only collides on the cuid PK, never on name) silently created a fresh row
-- for every artist that listed an already-known genre.
--
-- Steps:
--   1. Pick a canonical id per name (smallest id alphabetically — deterministic).
--   2. Repoint artist_genres rows at the canonical id, ignoring (artist_id,
--      genre_id) pairs that would collide with the canonical row.
--   3. Delete the now-unreferenced duplicate genre rows.
--   4. Recreate the genre table with UNIQUE(name).

BEGIN;

CREATE TEMPORARY TABLE genre_canonical AS
SELECT name, MIN(id) AS canonical_id
FROM genre
GROUP BY name;

-- Drop artist_genres rows that would violate UNIQUE(artist_id, genre_id) once
-- their genre_id is rewritten to the canonical one.
DELETE FROM artist_genres
WHERE id IN (
    SELECT ag.id
    FROM artist_genres ag
    JOIN genre g ON g.id = ag.genre_id
    JOIN genre_canonical c ON c.name = g.name
    WHERE ag.genre_id <> c.canonical_id
      AND EXISTS (
          SELECT 1 FROM artist_genres ag2
          WHERE ag2.artist_id = ag.artist_id
            AND ag2.genre_id = c.canonical_id
      )
);

-- Repoint the survivors at the canonical id.
UPDATE artist_genres
SET genre_id = (
    SELECT c.canonical_id
    FROM genre g
    JOIN genre_canonical c ON c.name = g.name
    WHERE g.id = artist_genres.genre_id
)
WHERE genre_id IN (
    SELECT g.id FROM genre g
    JOIN genre_canonical c ON c.name = g.name
    WHERE g.id <> c.canonical_id
);

-- Delete the now-unreferenced duplicates.
DELETE FROM genre
WHERE id NOT IN (SELECT canonical_id FROM genre_canonical);

DROP TABLE genre_canonical;

-- Recreate genre with UNIQUE(name). SQLite can't ADD CONSTRAINT, so we do the
-- table-rename dance.
CREATE TABLE genre_new (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    image VARCHAR(255)
);

INSERT INTO genre_new (id, name, description, image)
SELECT id, name, description, image FROM genre;

DROP TABLE genre;
ALTER TABLE genre_new RENAME TO genre;

COMMIT;
