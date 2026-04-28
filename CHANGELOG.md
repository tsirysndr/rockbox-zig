# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [2026.04.28-1]

### Added
- Real-time PCM loudness normalizer (`normalize_volume = true` in `settings.toml`) — RMS-based AGC with asymmetric attack/release, similar to Spotify's "Normalize Volume"; applied across all PCM sinks (SDL, FIFO, AirPlay, Squeezelite, UPnP, Chromecast, Snapcast TCP)
- `GET /player/volume` REST endpoint returning `{ volume, min, max }`
- `volume` GraphQL query returning live current volume with min/max range
- `useGetVolumeQuery` GraphQL hook in the web UI
- `get_current_volume()` gRPC client helper in the GPUI app

### Fixed
- Volume slider in GPUI mini-player now responds to mouse clicks (replaced plain `div` with `SeekBar` component)
- Volume slider in web UI now uses correct 0–100 range with explicit `min`/`max` on the MUI Slider
- `globalSettings.volume` in GraphQL now returns the live current volume via `rb::sound::current(0)` instead of a hardcoded `0`
- `VOLUME_MIN_DB` constant in GPUI corrected from `-74` to `-80` (SDL target range)
- Volume in GPUI loads the live value at startup via `SoundCurrent` gRPC instead of the stale saved setting
- `adjust_volume` now has audible effect on all non-SDL PCM sinks (FIFO, AirPlay, Squeezelite, UPnP, Chromecast, Snapcast TCP) — SW volume scaling (`pcm_copy_buffer`) was not being applied in any of these sinks

## [2026.04.28]

### Added
- Snapcast TCP PCM sink (`audio_output = "snapcast_tcp"`) — streams S16LE PCM directly to a Snapcast `tcp://` source; compatible with snapserver v0.35+
- Stream metadata forwarding for Snapcast TCP sink

### Fixed
- MPD `getvol` / `setvol` handlers now correctly map the Rockbox dB range (−80..0) to the MPD 0–100 scale

## [2026.04.27]

### Added
- TypeScript SDK (`@rockbox/sdk`) for controlling rockboxd from Node.js / browser applications
- Playlists UI in the web interface — create, edit, and manage saved and smart playlists
- Album art footer overlay shown on album cover hover

### Changed
- Web UI data layer migrated from Apollo Client to TanStack React Query
- Playlist modals rendered into document body via React portal (fixes z-index stacking issues)

## [2026.04.26]

### Added
- Chromecast PCM sink (`audio_output = "chromecast"`) — streams WAV over HTTP and controls playback via the Cast Media protocol
- UPnP/DLNA support: ContentDirectory media server (`upnp_server_enabled`), MediaRenderer:1 (`upnp_renderer_enabled`), and UPnP PCM sink (`audio_output = "upnp"`) with auto-renderer discovery
- Device picker UI in the GPUI and web mini-player — switch audio output (Rockbox built-in, AirPlay, Squeezelite, Chromecast) without restarting
- Multi-room AirPlay: `airplay_receivers` list in `settings.toml` supports sending to multiple RAOP receivers simultaneously
- Squeezelite multi-room PCM sink (`audio_output = "squeezelite"`) — Slim Protocol TCP server + HTTP PCM broadcast; supports unlimited concurrent squeezelite clients with independent reader cursors

### Fixed
- Duplicate Chromecast devices skipped during discovery
- Typesense search index initialised before the HTTP server accepts requests (avoids empty results on cold start)

## [2026.04.25]

### Added
- Saved playlists: create, rename, delete, and reorder tracks via gRPC, GraphQL, and REST APIs
- Smart playlists: rule-based auto-generated playlists with play-count and skip-count tracking
- Playlist search integration with Typesense
- `StreamLibrary` gRPC server-streaming RPC — pushes library updates to clients when a scan completes
- GPUI file browser: navigate the local filesystem and enqueue directories directly

### Fixed
- Now Playing widget in GPUI shows correctly when the app opens with a paused track (initial status fetched once at startup)
- Rocksky registration failures logged at `debug` level instead of `warn` to reduce noise
- Global Play/Pause keybind no longer fires when a text input field has focus
