CREATE INDEX IF NOT EXISTS idx_saved_playlist_tracks_playlist_pos
    ON saved_playlist_tracks(playlist_id, position);

CREATE INDEX IF NOT EXISTS idx_saved_playlist_tracks_track
    ON saved_playlist_tracks(track_id);
