-- Cache table for Last.fm `artist.getInfo` responses. Read from every
-- `artist_to_dto` call to enrich `Overview` / `Genres`; the row is
-- refreshed on demand from the detail handlers (`/Items/{id}`,
-- `/Users/{uid}/Items/{id}`, `/Artists/{name}`) when it's missing or
-- older than `ENRICHMENT_TTL_SECS`.
--
-- `artist_id` is the rockbox-library native id; MB / Last.fm identifiers
-- are stored for reference but the join back to the local library is by
-- native id.
CREATE TABLE IF NOT EXISTS jf_artist_enrichment (
    artist_id  TEXT PRIMARY KEY,     -- rockbox-library artist id
    mbid       TEXT,                 -- MusicBrainz artist id, when known
    bio        TEXT,                 -- summary or content — cleaned of "read more" links
    tags       TEXT,                 -- JSON-encoded Vec<String>
    image_url  TEXT,                 -- Last.fm image URL (may be a placeholder)
    fetched_at INTEGER NOT NULL      -- unix seconds
);
