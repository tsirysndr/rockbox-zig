pub enum RockboxCommand {
    Pause,
    Play(i64, i64),
    Resume,
    Next,
    Prev,
    FfRewind(i32),
    FlushAndReloadTracks,
    Stop,
    PlaylistResume,
    PlaylistResumeTrack,
}
