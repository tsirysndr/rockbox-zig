use crate::state::{PlaybackStatus, Track};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use std::sync::mpsc;
use std::time::Duration;

/// Commands forwarded from the OS media-control callbacks to the GPUI poll loop.
pub enum MediaCommand {
    Play,
    Pause,
    Toggle,
    Next,
    Prev,
    SeekTo(Duration),
}

/// Owns the souvlaki `MediaControls` handle and a channel for incoming OS commands.
///
/// Must be created on the main thread (macOS requires MPRemoteCommandCenter
/// registration on the main thread). The GPUI foreground poll loop — also on
/// the main thread — calls `drain_commands` and `update` each tick.
pub struct NowPlayingManager {
    controls: MediaControls,
    cmd_rx: mpsc::Receiver<MediaCommand>,
    /// Last track id pushed to the OS — guards against redundant metadata sends.
    last_track_id: String,
    /// Last cover URL — re-send metadata when art arrives after the track id.
    last_cover_url: Option<String>,
}

impl NowPlayingManager {
    /// Returns `None` if the OS media-control API is unavailable.
    pub fn new() -> Option<Self> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<MediaCommand>();

        let cfg = PlatformConfig {
            dbus_name: "org.rockbox.Rockbox",
            display_name: "Rockbox",
            hwnd: None,
        };

        let mut controls = MediaControls::new(cfg).ok()?;

        controls
            .attach(move |event: MediaControlEvent| {
                let cmd = match event {
                    MediaControlEvent::Play => Some(MediaCommand::Play),
                    MediaControlEvent::Pause => Some(MediaCommand::Pause),
                    MediaControlEvent::Toggle => Some(MediaCommand::Toggle),
                    MediaControlEvent::Next => Some(MediaCommand::Next),
                    MediaControlEvent::Previous => Some(MediaCommand::Prev),
                    MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                        Some(MediaCommand::SeekTo(pos))
                    }
                    _ => None,
                };
                if let Some(c) = cmd {
                    let _ = cmd_tx.send(c);
                }
            })
            .ok()?;

        Some(NowPlayingManager {
            controls,
            cmd_rx,
            last_track_id: String::new(),
            last_cover_url: None,
        })
    }

    /// Drain all pending OS media-key commands (non-blocking).
    pub fn drain_commands(&mut self) -> Vec<MediaCommand> {
        let mut out = Vec::new();
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            out.push(cmd);
        }
        out
    }

    /// Push the current playback state and track metadata to the OS.
    pub fn update(&mut self, track: Option<&Track>, status: PlaybackStatus, position: u64) {
        let track_id = track.map(|t| t.id.as_str()).unwrap_or("");
        // album_art is a bare filename served by rockboxd's cover HTTP server.
        let cover_url = track
            .and_then(|t| t.album_art.as_deref())
            .filter(|s| !s.is_empty())
            .map(|name| format!("http://localhost:6062/covers/{}", name));

        // Don't touch MPNowPlayingInfoCenter until we have (or previously had) a track.
        // Calling set_playback(Stopped) every 100ms during the startup window before
        // any queue data arrives causes macOS to deregister the app from Now Playing,
        // and a subsequent set_playback(Paused/Playing) then fails to re-show the widget.
        let had_track = !self.last_track_id.is_empty();
        let has_track = !track_id.is_empty();
        if !had_track && !has_track {
            return;
        }

        let track_changed = track_id != self.last_track_id;
        let cover_changed = cover_url != self.last_cover_url;

        // souvlaki's macOS set_playback_metadata() replaces nowPlayingInfo with a fresh
        // dict (no elapsed time). set_playback_progress() reads the existing dict and
        // merges elapsed time into it. Calling set_metadata BEFORE set_playback in the
        // same tick ensures the progress merge sees the fresh metadata dict, so the
        // final nowPlayingInfo always contains both metadata and elapsed time.
        if track_changed || cover_changed {
            self.last_track_id = track_id.to_string();
            self.last_cover_url = cover_url.clone();

            if has_track {
                let meta = track
                    .map(|t| MediaMetadata {
                        title: if t.title.is_empty() { None } else { Some(t.title.as_str()) },
                        artist: if t.artist.is_empty() { None } else { Some(t.artist.as_str()) },
                        album: if t.album.is_empty() { None } else { Some(t.album.as_str()) },
                        cover_url: cover_url.as_deref(),
                        duration: if t.duration > 0 {
                            Some(Duration::from_secs(t.duration))
                        } else {
                            None
                        },
                    })
                    .unwrap_or_default();
                let _ = self.controls.set_metadata(meta);
            }
        }

        let progress = Some(MediaPosition(Duration::from_secs(position)));
        let playback = match status {
            PlaybackStatus::Playing => MediaPlayback::Playing { progress },
            PlaybackStatus::Paused => MediaPlayback::Paused { progress },
            PlaybackStatus::Stopped => MediaPlayback::Stopped,
        };
        let _ = self.controls.set_playback(playback);
    }
}
