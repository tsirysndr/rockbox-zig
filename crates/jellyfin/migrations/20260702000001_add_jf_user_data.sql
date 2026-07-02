-- Per-item user data for the Jellyfin API — populates `UserItemDataDto`
-- fields that don't map cleanly to existing rockbox-library state
-- (play_count, playback position, played flag, likes, rating).
--
-- `IsFavorite` still lives in `jf_favorites`; this table is purely for
-- the numeric / boolean fields that Jellyfin clients round-trip through
-- POST /UserItems/{id}/UserData.
--
-- For tracks we merge with `track_stats` (rockbox-playlists) on read
-- so playback counters recorded by the audio engine surface here
-- without an extra sync step. Writes still land in `jf_user_data` so
-- Jellyfin-side edits stay isolated from the engine's own bookkeeping.
CREATE TABLE IF NOT EXISTS jf_user_data (
    kind                    TEXT NOT NULL,       -- 'track'|'album'|'artist'|'playlist'
    native_id               TEXT NOT NULL,
    played                  INTEGER NOT NULL DEFAULT 0,
    play_count              INTEGER NOT NULL DEFAULT 0,
    playback_position_ticks INTEGER NOT NULL DEFAULT 0,
    last_played_at          TEXT,                -- ISO-8601 or NULL
    likes                   INTEGER,             -- nullable tri-state: 0|1|NULL
    rating                  REAL,                -- 0.0..10.0 or NULL
    updated_at              TEXT NOT NULL,
    PRIMARY KEY (kind, native_id)
);
CREATE INDEX IF NOT EXISTS idx_jf_user_data_kind ON jf_user_data(kind);
