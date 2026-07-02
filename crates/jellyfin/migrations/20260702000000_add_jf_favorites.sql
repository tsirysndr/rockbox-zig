-- Jellyfin favorites — source of truth for `IsFavorite` on all four
-- kinds Jellyfin models: tracks, albums, artists, and playlists.
--
-- For tracks and albums we ALSO mirror writes to rockbox-library's
-- existing `favourites` table so smart-playlist rules (`is_liked`) and
-- the Subsonic bridge stay in sync. Reads accept a row from either
-- table as "favorited" so a like added elsewhere still surfaces here.
CREATE TABLE IF NOT EXISTS jf_favorites (
    kind         TEXT NOT NULL,      -- 'track' | 'album' | 'artist' | 'playlist'
    native_id    TEXT NOT NULL,      -- rockbox-library / rockbox-playlists id
    favorited_at TEXT NOT NULL,      -- ISO-8601 timestamp
    PRIMARY KEY (kind, native_id)
);
CREATE INDEX IF NOT EXISTS idx_jf_favorites_kind ON jf_favorites(kind);
