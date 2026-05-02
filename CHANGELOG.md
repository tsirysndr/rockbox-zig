# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [2026.05.02-1]

### Added
- Mintlify documentation site under `mintlify/` with the Linden theme; OpenAPI spec regenerated and ASCII architecture diagrams replaced with `CardGroup` components
- Linux-specific window controls (minimize / maximize / close) in the GPUI titlebar — macOS/Windows continue to use native traffic-light controls

### Changed
- GPUI titlebar drag areas now call `window.start_window_move()` from an `on_mouse_down` handler instead of relying on `WindowControlArea::Drag`, fixing window dragging on Linux/X11
- Debian and RPM packages now declare XKB/XCB build dependencies (`libxkbcommon-dev`, `libxkbcommon-x11-dev`, `libxcb1-dev`, `libxcb-render0-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`); README updated with the matching install instructions
- Debian package version bumped to `2026.05.02-1`

### Fixed
- GPUI app no longer fails to build on Linux: `souvlaki` is now a non-Linux-only dependency and `NowPlayingManager` ships a no-op Linux stub, since the OS media-control APIs souvlaki targets are not available there

## [2026.05.02]

### Added
- New SDKs for controlling rockboxd from Python, Ruby, Elixir, Gleam, and Clojure (`sdk/python/`, `sdk/ruby/`, `sdk/elixir/`, `sdk/gleam/`, `sdk/clojure/`) — each ships with examples covering playback, queue, library search, saved/smart playlists, volume/EQ, browse, devices, Bluetooth, and plugins
- TypeScript SDK gains 15 runnable examples (`sdk/typescript/examples/`) plus a Bluetooth API (`api/bluetooth.ts`) and a `getVolume` / `VolumeInfo` endpoint on `api/sound.ts`
- TS SDK types extended with `browse.displayName` and `album.copyrightMessage`

### Fixed
- HTTP/remote tracks now hydrate `Mp3Entry` metadata (title, artist, album, duration, etc.) from the DB `Track` record in the playlist handlers when Rockbox cannot read tags locally
- GPUI Library page: text truncation and unexpected overflow on likes and track rows resolved by adding `min_w_0` / `flex_shrink_0` to the flex containers
- Regenerated tonic/prost UPnP bindings under `crates/upnp/src/api/`

## [2026.05.01]

### Added
- Bluetooth button in the GPUI mini-player — shown when Bluetooth is available; opens the device picker and fetches paired devices on toggle
- Cover URLs in GPUI now follow the active server via `get_covers_base()` instead of the hardcoded `http://localhost:6062/covers/` base

### Changed
- HTTP server (`crates/server`) migrated from a custom request/response layer to **Actix-web** — handlers now accept `web::Data`, `web::Path`, and `web::Query` and return `actix_web::Result<HttpResponse>`; blocking C FFI work is offloaded to `web::block`
- Tokio runtimes for the controls and MPD servers are now shared via `OnceLock` instead of being created per-thread, reducing overhead and avoiding nested-runtime panics
- `RLIMIT_NOFILE` is raised to 4 096 at startup on Unix to accommodate large music libraries

### Fixed
- Audio `stop` and `pause` are now non-blocking — they use `audio_queue_post` so they can safely be called from any OS thread; `audio_hard_stop` posts `Q_AUDIO_STOP` with `data=2` and the audio thread frees `audiobuf_handle` itself, preventing cross-thread frees
- Blocking C FFI calls in playlist handlers run on `web::block` threads to avoid starving Actix worker threads and prevent nested tokio/reqwest blocking contexts
- Live metadata lookups are skipped for HTTP tracks; Rockbox's own UPnP renderers are excluded from the UPnP device list
- Bluetooth availability check uses `fetchGlobalStatus()` (gRPC `GetGlobalStatus`) instead of `getDevices()` to avoid spurious `UNIMPLEMENTED` errors on probe
- Bluetooth availability is now polled in a background task and updated via `std::sync::mpsc` to avoid cross-runtime waker issues when bridging Tokio → GPUI
- `observe_global` registrations in GPUI now call `.detach()` instead of silently dropping the subscription handle
- RFC3339 datetime migration — a SQL migration normalises `NULL`/blank and `YYYY-MM-DD HH:MM:SS` timestamps in the library database to RFC3339 so SQLx `DateTime<Utc>` decoding no longer fails
- Favourites queries now use `INNER JOIN` and filter out empty-string IDs, excluding bogus entries from results
- mDNS scanning now prefers IPv4 addresses (192.168 → 10 → others) and selects the best non-loopback/link-local address so multiple records for the same host coalesce correctly
- `println!`/`eprintln!` diagnostics in `crates/controls` and `crates/mpd` replaced with `tracing::error!`
- macOS app listens for server-change notifications and restarts streaming, re-fetches settings, device state, and Bluetooth state on server switch

## [2026.04.31]

### Fixed
- mDNS device ID is now persisted across restarts — a 64-bit hex ID is generated once and cached in `~/.config/rockbox.org/device-id`, so the registered mDNS service name remains stable between daemon restarts instead of changing on every launch

## [2026.04.30]

### Added
- Bluetooth device support in the GPUI and web UIs — list paired/discovered devices, connect and disconnect directly from the device picker
- mDNS-based server discovery and runtime server switching — `scan_mdns()` in the daemon registers itself via mDNS; the GPUI app and macOS app gain a Server Picker UI that enumerates nearby `rockboxd` instances and switches without restart; a notification triggers one-shot syncs to re-run on server change
- UPnP album art saved for remote tracks — `album_art_uri` is returned from UPnP directory listings; `save_audio_metadata` downloads and caches the cover when no embedded art is present; remote metadata is persisted concurrently (semaphore-limited) without blocking C/FFI
- `copyright_message` field on the `Album` GraphQL type, displayed in `AlbumDetails` alongside a formatted release date
- Typesense bundled in the Docker image — the Dockerfile now pulls the typesense image and copies `typesense-server` into the final image

## [2026.04.29-2]

### Added
- Bluetooth speaker management commands in the `rockbox` CLI (`bluetooth scan`, `bluetooth devices`, `bluetooth connect <address>`, `bluetooth disconnect <address>`) — Linux only, talks to a running `rockboxd` via gRPC
- Bluetooth GraphQL resolvers (`bluetoothDevices` query, `bluetoothScan` / `bluetoothConnect` / `bluetoothDisconnect` mutations) now call `rockbox-bluetooth` directly instead of going through the HTTP server — eliminates an extra round-trip on Linux

### Fixed
- `BluetoothService` gRPC RPC renamed from `Connect` to `ConnectDevice` to avoid a name collision with tonic's auto-generated transport `connect` constructor, which caused a compile error (`duplicate definitions with name connect`)

## [2026.04.29-1]

### Fixed
- macOS app Files view: navigating from the root into Music no longer yields an empty list — `.task` ID now encodes both mode and path so a mode change with a nil path correctly triggers a reload
- macOS app device picker: now lists all output devices (including the current one, marked with a checkmark) instead of only non-current devices; added `snapcast` icon/colour entry
- macOS app device picker: no longer shows a loading spinner on open when devices were already preloaded at startup — `refresh()` only sets `isLoading` when the device list is empty

## [2026.04.29]

### Added
- UPnP device browsing in the Files view — queue and play tracks directly from any UPnP/DLNA media server on the local network

### Fixed
- HTTP stream (`netstream`) no longer permanently breaks after a failed seek: `seek_to()` now only replaces the active response on success, so a failed Range request leaves the stream readable at the current position
- Small forward seeks (≤ 128 KB) in HTTP streams are now satisfied by skipping bytes in the existing response body instead of issuing a new Range request, avoiding unnecessary round-trips during codec metadata parsing
- Buffering: `TYPE_ID3` handles for remote tracks that fail to open now send `BUFFER_EVENT_FINISHED` with an empty `mp3entry` instead of silently never posting `Q_AUDIO_FINISH_LOAD_TRACK`, which caused the track-loading chain to stall on playlist restore with many queued UPnP tracks
- Web UI Files view: Music and UPnP Devices row icons no longer disappear on hover — CSS selector changed from descendant (` `) to direct-child (`>`) combinator so the `.no-play` guard is respected

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
