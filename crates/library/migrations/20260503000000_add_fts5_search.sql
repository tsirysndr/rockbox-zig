-- FTS5 full-text search indexes mirroring the searchable columns of
-- track / album / artist / saved_playlists / smart_playlists.
--
-- These tables and triggers are always created so that toggling the
-- `fts5` cargo feature on a consumer crate does not require a manual
-- reindex. When the feature is off (default), Typesense is used and these
-- indexes are simply maintained but never queried.
--
-- Tokenizer: porter for stemming + unicode61 with diacritics removed,
-- so "beyonce" matches "Beyoncé".

CREATE VIRTUAL TABLE IF NOT EXISTS track_fts USING fts5(
    id UNINDEXED,
    title,
    artist,
    album,
    album_artist,
    composer,
    genre,
    path,
    tokenize = "porter unicode61 remove_diacritics 2"
);

CREATE TRIGGER IF NOT EXISTS track_fts_ai AFTER INSERT ON track BEGIN
    INSERT INTO track_fts(id, title, artist, album, album_artist, composer, genre, path)
    VALUES (
        new.id,
        COALESCE(new.title, ''),
        COALESCE(new.artist, ''),
        COALESCE(new.album, ''),
        COALESCE(new.album_artist, ''),
        COALESCE(new.composer, ''),
        COALESCE(new.genre, ''),
        COALESCE(new.path, '')
    );
END;

CREATE TRIGGER IF NOT EXISTS track_fts_ad AFTER DELETE ON track BEGIN
    DELETE FROM track_fts WHERE id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS track_fts_au AFTER UPDATE ON track BEGIN
    DELETE FROM track_fts WHERE id = old.id;
    INSERT INTO track_fts(id, title, artist, album, album_artist, composer, genre, path)
    VALUES (
        new.id,
        COALESCE(new.title, ''),
        COALESCE(new.artist, ''),
        COALESCE(new.album, ''),
        COALESCE(new.album_artist, ''),
        COALESCE(new.composer, ''),
        COALESCE(new.genre, ''),
        COALESCE(new.path, '')
    );
END;

-- Backfill any tracks already present (idempotent).
INSERT INTO track_fts(id, title, artist, album, album_artist, composer, genre, path)
SELECT
    t.id,
    COALESCE(t.title, ''),
    COALESCE(t.artist, ''),
    COALESCE(t.album, ''),
    COALESCE(t.album_artist, ''),
    COALESCE(t.composer, ''),
    COALESCE(t.genre, ''),
    COALESCE(t.path, '')
FROM track t
WHERE NOT EXISTS (SELECT 1 FROM track_fts f WHERE f.id = t.id);

CREATE VIRTUAL TABLE IF NOT EXISTS album_fts USING fts5(
    id UNINDEXED,
    title,
    artist,
    label,
    tokenize = "porter unicode61 remove_diacritics 2"
);

CREATE TRIGGER IF NOT EXISTS album_fts_ai AFTER INSERT ON album BEGIN
    INSERT INTO album_fts(id, title, artist, label)
    VALUES (
        new.id,
        COALESCE(new.title, ''),
        COALESCE(new.artist, ''),
        COALESCE(new.label, '')
    );
END;

CREATE TRIGGER IF NOT EXISTS album_fts_ad AFTER DELETE ON album BEGIN
    DELETE FROM album_fts WHERE id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS album_fts_au AFTER UPDATE ON album BEGIN
    DELETE FROM album_fts WHERE id = old.id;
    INSERT INTO album_fts(id, title, artist, label)
    VALUES (
        new.id,
        COALESCE(new.title, ''),
        COALESCE(new.artist, ''),
        COALESCE(new.label, '')
    );
END;

INSERT INTO album_fts(id, title, artist, label)
SELECT a.id, COALESCE(a.title, ''), COALESCE(a.artist, ''), COALESCE(a.label, '')
FROM album a
WHERE NOT EXISTS (SELECT 1 FROM album_fts f WHERE f.id = a.id);

CREATE VIRTUAL TABLE IF NOT EXISTS artist_fts USING fts5(
    id UNINDEXED,
    name,
    bio,
    tokenize = "porter unicode61 remove_diacritics 2"
);

CREATE TRIGGER IF NOT EXISTS artist_fts_ai AFTER INSERT ON artist BEGIN
    INSERT INTO artist_fts(id, name, bio)
    VALUES (new.id, COALESCE(new.name, ''), COALESCE(new.bio, ''));
END;

CREATE TRIGGER IF NOT EXISTS artist_fts_ad AFTER DELETE ON artist BEGIN
    DELETE FROM artist_fts WHERE id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS artist_fts_au AFTER UPDATE ON artist BEGIN
    DELETE FROM artist_fts WHERE id = old.id;
    INSERT INTO artist_fts(id, name, bio)
    VALUES (new.id, COALESCE(new.name, ''), COALESCE(new.bio, ''));
END;

INSERT INTO artist_fts(id, name, bio)
SELECT a.id, COALESCE(a.name, ''), COALESCE(a.bio, '')
FROM artist a
WHERE NOT EXISTS (SELECT 1 FROM artist_fts f WHERE f.id = a.id);

-- Single FTS table covers both saved_playlists and smart_playlists; the
-- `is_smart` column tells the query layer which source table to pull
-- the document from when reconstructing results.
CREATE VIRTUAL TABLE IF NOT EXISTS playlist_fts USING fts5(
    id UNINDEXED,
    is_smart UNINDEXED,
    name,
    description,
    tokenize = "porter unicode61 remove_diacritics 2"
);

CREATE TRIGGER IF NOT EXISTS saved_playlist_fts_ai AFTER INSERT ON saved_playlists BEGIN
    INSERT INTO playlist_fts(id, is_smart, name, description)
    VALUES (new.id, 0, COALESCE(new.name, ''), COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS saved_playlist_fts_ad AFTER DELETE ON saved_playlists BEGIN
    DELETE FROM playlist_fts WHERE id = old.id AND is_smart = 0;
END;

CREATE TRIGGER IF NOT EXISTS saved_playlist_fts_au AFTER UPDATE ON saved_playlists BEGIN
    DELETE FROM playlist_fts WHERE id = old.id AND is_smart = 0;
    INSERT INTO playlist_fts(id, is_smart, name, description)
    VALUES (new.id, 0, COALESCE(new.name, ''), COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS smart_playlist_fts_ai AFTER INSERT ON smart_playlists BEGIN
    INSERT INTO playlist_fts(id, is_smart, name, description)
    VALUES (new.id, 1, COALESCE(new.name, ''), COALESCE(new.description, ''));
END;

CREATE TRIGGER IF NOT EXISTS smart_playlist_fts_ad AFTER DELETE ON smart_playlists BEGIN
    DELETE FROM playlist_fts WHERE id = old.id AND is_smart = 1;
END;

CREATE TRIGGER IF NOT EXISTS smart_playlist_fts_au AFTER UPDATE ON smart_playlists BEGIN
    DELETE FROM playlist_fts WHERE id = old.id AND is_smart = 1;
    INSERT INTO playlist_fts(id, is_smart, name, description)
    VALUES (new.id, 1, COALESCE(new.name, ''), COALESCE(new.description, ''));
END;

INSERT INTO playlist_fts(id, is_smart, name, description)
SELECT p.id, 0, COALESCE(p.name, ''), COALESCE(p.description, '')
FROM saved_playlists p
WHERE NOT EXISTS (SELECT 1 FROM playlist_fts f WHERE f.id = p.id AND f.is_smart = 0);

INSERT INTO playlist_fts(id, is_smart, name, description)
SELECT p.id, 1, COALESCE(p.name, ''), COALESCE(p.description, '')
FROM smart_playlists p
WHERE NOT EXISTS (SELECT 1 FROM playlist_fts f WHERE f.id = p.id AND f.is_smart = 1);
