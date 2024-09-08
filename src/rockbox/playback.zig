const c = @cImport(
    @cInclude("metadata.h"),
);

extern fn audio_pause() void;
extern fn audio_play(elapsed: u64, offset: u64) void;
extern fn audio_resume() void;
extern fn audio_next() void;
extern fn audio_prev() void;
extern fn audio_ff_rewind(newtime: i64) void;
extern fn audio_next_track() *c.mp3entry;
extern fn audio_status() i32;
extern fn audio_current_track() *c.mp3entry;

pub fn audioPlay(elapsed: u64, offset: u64) void {
    audio_play(elapsed, offset);
}

pub fn audioPause() void {
    audio_pause();
}

pub fn audioResume() void {
    audio_resume();
}

pub fn audioNext() void {
    audio_next();
}

pub fn audioPrev() void {
    audio_prev();
}

pub fn audioFfRewind(newtime: i64) void {
    audio_ff_rewind(newtime);
}

pub fn audioNextTrack() *c.mp3entry {
    return audio_next_track();
}

pub fn audioStatus() i32 {
    return audio_status();
}

pub fn audioCurrentTrack() *c.mp3entry {
    return audio_current_track();
}

pub fn audioFlushAndReloadTracks() void {}

pub fn audioGetFilePos() void {}

pub fn audioHardStop() void {}
