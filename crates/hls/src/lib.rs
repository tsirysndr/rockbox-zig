//! Standalone HLS / DASH client for Rockbox.
//!
//! Fully isolated from Rockbox's playback engine: no pcmbuf, no codec
//! dispatcher, no kernel threads. The pipeline is pure Rust:
//!
//!     URL (.m3u8 / .mpd)
//!         → manifest parser (m3u8-rs / dash-mpd)
//!             → segment fetcher (reqwest, concurrent prefetch)
//!                 → demuxer (symphonia: fMP4 + MPEG-TS → AAC frames)
//!                     → decoder (fdk-aac → S16LE PCM)
//!                         → output: pcm_external_write(addr, size)
//!                                   → forwards to sinks[cur_sink]->ops.play()
//!
//! The output stage doesn't touch Rockbox's audio buffer at all — it pushes
//! decoded PCM straight into whichever PCM sink the user has currently
//! selected (`audio_output = "cmaf" | "airplay" | "snapcast_tcp" | …`). That
//! makes "phone A broadcasts CMAF / phone B consumes it" trivially symmetric,
//! and lets the same HLS player re-broadcast as CMAF, AirPlay-cast, etc.

mod decoder;
mod demux;
mod fetcher;
mod manifest;
mod output;
mod player;

pub use manifest::{is_hls_or_dash_url, ManifestKind};
pub use player::{
    is_active as player_is_active, pause as player_pause, play as player_play,
    remote_api_base as player_remote_api_base, resume as player_resume,
    status_json as player_status_json, stop as player_stop, Player, PlayerState,
};

/// Force this crate's symbols into the surrounding staticlib.
#[doc(hidden)]
pub fn _link_hls() {}

// FFI exports live in player.rs alongside the Player implementation.
