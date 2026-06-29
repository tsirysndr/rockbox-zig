-- Jellyfin sidecar tables. Run on every startup; harmless on re-run.

CREATE TABLE IF NOT EXISTS jellyfin_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS jellyfin_tokens (
    token       TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL,
    device_id   TEXT,
    device_name TEXT,
    client      TEXT,
    created_at  TEXT NOT NULL
);

-- Reverse lookup from a dashed-UUID GUID we emit to Jellyfin clients back
-- to the native rockbox-library id (artist/album/track UUID strings).
CREATE TABLE IF NOT EXISTS jf_guids (
    guid      TEXT PRIMARY KEY,
    kind      TEXT NOT NULL,
    native_id TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_jf_guids_native ON jf_guids(kind, native_id);
