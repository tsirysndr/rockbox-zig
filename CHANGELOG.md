# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [2026.05.28]

### Added
- `netstream`: background prefetch thread per HTTP stream — a dedicated thread fills a 2 MB `VecDeque` buffer from the network so `rb_net_read` drains in sub-µs without ever blocking on TCP; forward seeks within the prefetch window consume buffered bytes without issuing a new `Range` request; backward or large (>2 MB) forward seeks issue a `Range` request and replace the reader thread atomically

### Fixed
- HTTP streaming: removed reqwest total-request timeout (only `connect_timeout` 15 s remains) — the previous 30 s deadline killed large remote files mid-stream; `read_as_file()` reverted to a retry-loop that fills the full requested buffer
- Buffering interleaving: `fill_buffer()` now passes `BUFFERING_DEFAULT_FILECHUNK` instead of `0` when a second handle has remaining data, so next-track pre-buffering round-robins with the current track instead of monopolising the buffering thread and starving the ring buffer
- HTTP pre-buffering cutting current-track playback: `buffer_handle()` caps HTTP handles to one `BUFFERING_DEFAULT_FILECHUNK` per call; `streamfd.c` replaces per-chunk `fprintf(stderr, …)` with `logf()` (compiled out in production) to eliminate hundreds of blocking `write(2)` syscalls per track
- Expo: Navidrome cover art now appears in the miniplayer, full-screen player, and queue when playing ND HTTP streams — `coverArtUrlFromStreamUrl()` added to `navidrome-client.ts` reconstructs a `getCoverArt` URL from the `id`, `u`, `t`, `s` parameters embedded in the stream URL; used as a fallback in `trackFromProto` when `album_art` is empty

## [2026.05.27]

### Added
- Navidrome / Subsonic support in the macOS Swift app — `NavidromeService` (Subsonic API client with MD5 token auth), `NavidromeManager` (multi-server persistence, active server switching, optimistic star toggling, cover art derivation from stream URLs), `NdResponseCache` (stale-while-revalidate actor cache, 30 min fresh TTL, 24 h eviction), `NdLibraryView` (Albums / Artists / Songs / Liked / Playlists sections with infinite-scroll pagination), `NdSongRowView` (track art toggle, hover play, star button, Play Next/Last + Go to Album/Artist context menu), `NdAlbumDetailView`, `NdArtistDetailView`, `NdPlaylistDetailView` (Play / Shuffle), and search integration (when a Navidrome server is active, `search3` replaces local gRPC search with ND artist circles, album cards, and song rows)

### Removed
- PCM volume normalizer (`pcm_normalizer.c`, `pcm_normalizer.h`, Rust bindings, settings field, docs) — superseded by ReplayGain perceived-loudness normalisation

### Fixed
- Expo: `AbortSignal.timeout()` replaced with `AbortController` + `setTimeout` in `navidrome-client.ts` — `AbortSignal.timeout` is absent in some Hermes / React Native versions and was silently swallowing timeouts, making every fetch return `null`; switched to `md5` npm package (removed inline implementation); set `NSAllowsArbitraryLoads=true` in iOS `infoPlist` to unblock HTTPS servers that do not meet strict ATS TLS requirements
- Expo ND album detail: mirror local `album/[id].tsx` hero layout — blurred background image (`blurRadius=40`) + dark gradient overlay + art shadow + scale/fade scroll animation + sticky header title fade-in; cover art URL now uses a stable salt derived from credentials so `expo-image`'s disk cache is not busted on every render
- Expo ND detail screens: cover art now renders correctly by placing computed dimensions on a parent `View` and giving `Image` `className="w-full h-full"` so NativeWind owns the style; track rows in album and playlist detail screens now include a `TrackMenuButton` "…" context menu
- GPUI: Navidrome cover art is now derived directly from the stream URL parameters (`id`, `u`, `t`, `s`) instead of requiring an active server connection, eliminating blank album art when playback starts before the ND panel is connected; removes the `PENDING_COVER_ART` staging mutex and the async `getSong` round-trip
- macOS Now Playing / `MPNowPlayingInfoCenter`: cover art priority corrected — `coverArtUrl(forStreamUrl:)` is now tried first (returns `nil` for local tracks), then falls back to `albumArt`; the previous order always hit the `albumArt` branch even when it pointed at an empty path, so Navidrome tracks showed no artwork in the system Now Playing widget
- CI: Android firmware build workflows now delete `make.dep` before `make lib` to force a fresh dependency scan after prefix-restore cache hits that carry stale header dependencies (e.g. the `pcm_normalizer.h` removal)

## [2026.05.25]

### Added
- Subsonic / Navidrome API compatibility server on port **4533** — any client that works with Navidrome (Cassette, Symfonium, DSub, Ultrasonic, Substreamer, Clementine, Sublime Music, …) can browse and stream music from `rockboxd` without additional setup; enabled by adding `subsonic_username` and `subsonic_password` to `settings.toml`
- Implemented endpoints: `ping`, `getUser`, `getMusicFolders`, `getScanStatus`, `startScan`, `getArtists`, `getArtist`, `getAlbum`, `getSong`, `getIndexes`, `getMusicDirectory`, `getGenres`, `getSongsByGenre`, `getAlbumList` / `getAlbumList2`, `getRandomSongs`, `getStarred` / `getStarred2`, `stream` (with HTTP `Range` / seek support), `download`, `getCoverArt`, `scrobble`, `getNowPlaying`, `updateNowPlaying`, `search2` / `search3`, `getPlaylists`, `getPlaylist`, `createPlaylist`, `updatePlaylist`, `deletePlaylist`, `star` / `unstar` (mirrored to Rocksky), `getArtistInfo` / `getArtistInfo2`, `getAlbumInfo` / `getAlbumInfo2`, `getSimilarSongs` / `getSimilarSongs2`, `getTopSongs`, `getLyrics`
- Auth: MD5 token mode (`t` + `s`) and plaintext / `enc:<hex>` mode (`p`)
- `getCoverArt` resolves bare filenames to `~/.config/rockbox.org/covers/` and proxies Rocksky HTTP URLs for artist images
- Mintlify docs page `mintlify/clients/subsonic.mdx` covering setup, auth modes, all endpoints, recommended clients, cover art IDs, and range-request support

## [2026.05.17]

### Added
- Web UI mobile layout — bottom-tab navigation bar, persistent mini-player dock, and a full-screen player modal; mirrors the Expo mobile app information architecture on small viewports

### Fixed
- Web UI: resuming a paused track now calls `resume` instead of restarting the track from the beginning — `useResumePlaylist` now scopes the playlist-reload logic to `status === 0` (stopped) only, preserves `nowPlaying` fields while paused, and fixes an `onPause` timeout that was permanently locking subscription updates after any pause
- Bluetooth: adapter is powered on before listing paired devices or disconnecting, preventing `BluetoothError::NotPowered` on adapters that idle to off

## [2026.05.15]

### Added
- Plex Media Server browsing via `plex://` scheme — mDNS discovery (`_plexmediasvr._tcp.local.`), token-in-URL auth, full library / playlist / album / artist / track navigation
- Jellyfin Media Server browsing via `jellyfin://` scheme — manual server entry, API-key auth, full content hierarchy browsing
- Navidrome Media Server browsing via `navidrome://` scheme — manual server entry, MD5 token auth (Subsonic API), `getIndexes` + `getMusicDirectory` browsing
- Kodi/XBMC Media Server browsing via `kodi://` scheme — JSON-RPC API, library browsing for audio albums, artists, genres, and tracks
- Expo mobile app: Plex, Jellyfin, Navidrome, and Kodi server browsing surfaced in the Files tab alongside the existing local filesystem view
- WASM browser build: settings API (persist EQ / DSP / volume / crossfade to in-memory config), playlist persistence across reloads, `rb_set_repeat` export (repeat off / all / one / shuffle)
- Real-time DSP/EQ API exposed over HTTP, gRPC, and GraphQL — `setEq` mutation with `enabled`, `precut`, and per-band `cutoff`/`Q`/`gain` fields; backed by `dsp_set_eq_coefs()` called directly on the audio thread to avoid audible cuts

### Changed
- Docker base images upgraded from Debian bookworm → trixie across all three Dockerfiles; Rust base image bumped from 1.94 → 1.95
- Nix flake now builds only `rockboxd` (removed unused outputs)
- `settings.toml` example updated to document the new media-server `audio_output` entries

### Fixed
- WASM: `seek`, crossfade, bass/treble DSP controls now apply correctly; real-time events (position, track change) fire reliably; crossfade mode change posts `Q_AUDIO_REMAKE_AUDIO_BUFFER` only when audio is playing to avoid an audible cut when stopped
- WASM: EQ real-time application and persistence — coefficient updates call `dsp_set_eq_coefs()` in the `wasm_cmd` handler without posting `REMAKE`; band gain multiplied by 10 (tenths of dB) before passing to `rb_set_eq_band`
- WASM: EQ cutoff and Q values now match the preset layout (Q 7.0, 10-band display) after correcting the unit conversion in `web/rockbox.js`
- Dithering, Auditory Frequency Resolution (AFR), and Perceptual Bass Enhancement (PBE) controls in the web UI now reflect changes immediately — `GlobalSettings` mutations now call the corresponding DSP setters and trigger `tracing`-level log output

## [2026.05.09]

### Fixed
- DSP compressor divide-by-zero crash on x86_64 (`SIGFPE` in `get_att_rls_coeff`) — added `release > 0` guard in `compressor_update()` mirroring the existing `attack > 0` guard; ARM64 silently returned 0 on integer divide-by-zero while x86_64 faulted; also added function-level guards in `get_att_rls_coeff` and `get_lpf_coeff` for zero `rc`/`fs`/`rc_units` parameters, and an early `fs <= 0` return in `compressor_update` for uninitialised output frequency
- Startup hang on second+ launch — FTS5 backfill `WHERE NOT EXISTS (SELECT 1 FROM fts_table f WHERE f.id = t.id)` forced an O(N) full scan per row (O(N²) total) because `id` is `UNINDEXED` in FTS5; replaced all four backfill INSERTs with an uncorrelated `WHERE NOT EXISTS (SELECT 1 FROM fts_table)` which SQLite short-circuits at the first row (O(1) for non-empty tables)
- Library startup blocked indefinitely on repeated runs — SQLx hangs when re-executing `CREATE VIRTUAL TABLE IF NOT EXISTS` on an existing FTS5 virtual table; fixed by checking `sqlite_master` before the migration and skipping it entirely if `track_fts` already exists; same guard added for `dedupe_genres` (checks `UNIQUE` constraint on `genre` table)
- FTS5 and `dedupe_genres` migrations ran in slow DELETE journal mode — `PRAGMA journal_mode=WAL` was set only after all migrations; moved to `SqliteConnectOptions::journal_mode(Wal)` so WAL is active from the first connection
- FTS5 index migration moved to a background `tokio::spawn` task so startup is non-blocking; `dedupe_genres` (schema DDL) remains synchronous with an O(1) skip guard
- cpal PCM sink: audible silence gap at the start of every track on Linux — `sink_dma_start()` previously stored the first chunk in `pcm_data`/`pcm_size` and then called `pthread_create`, leaving the ring empty for the 1–5 ms thread-creation window; fixed by pushing the first chunk synchronously via `pcm_cpal_push()` before spawning the writer thread so the ring is pre-filled when `running=true` is set; the writer thread now picks up from chunk 2 onwards; also added `!r.running` early-exit to the f32 cpal callback (mirrors the existing i16 guard) and reset resampler state (`cur_valid = false`, `phase = 0`) in `pcm_cpal_start()` to prevent interpolation artefacts from the tail of the previous track

## [2026.05.05]

### Added
- Headless host target and `cpal` PCM sink (`audio_output = "cpal"`) — runs Rockbox without SDL on any OS audio backend (ALSA, CoreAudio, WASAPI, JACK) via CPAL; build with `scripts/build-headless.sh`; documented in `HEADLESS.md`
- Genres API — gRPC, GraphQL, REST, and CLI endpoints to list genres, fetch tracks by genre, and add genre-based smart playlist rules; genre deduplication SQL migration bundled
- Disc/track number support in the Expo mobile album view — `TrackList` component sorts by (disc, track) and renders disc-section headers for multi-disc releases; `proto track_number`/`disc_number` fields mapped through to the UI
- Pull-to-refresh / rescan in the Expo library tab

### Changed
- CI workflows, macOS build scripts, Dockerfile, and `install.sh` streamlined — significant reduction in duplication and overall build time
- Android `cdylib` option now available in the `tools/configure` interactive menu

### Fixed
- M4A/AAC files decode silently in `CODECS_STATIC` builds — dead-write elimination in `libm4a/demux.c` was optimizing away box-parsing reads; replaced with live-return readers (`stream_read_uint*` + `stream_skip`)
- macOS linker: `Security.framework` explicitly linked in `zig/build.zig` to resolve missing symbol errors when using macOS Security APIs
- Expo mobile app re-establishes gRPC subscriptions when the app returns to the foreground (`reconnectEpoch` bump + `reapplyServerUrl()`)

## [2026.05.03]

### Added
- Mintlify documentation site under `mintlify/` with the Linden theme; OpenAPI spec regenerated and ASCII architecture diagrams replaced with `CardGroup` components
- Linux-specific window controls (minimize / maximize / close) in the GPUI titlebar — macOS/Windows continue to use native traffic-light controls

### Changed
- GPUI titlebar drag areas now call `window.start_window_move()` from an `on_mouse_down` handler instead of relying on `WindowControlArea::Drag`, fixing window dragging on Linux/X11
- Debian and RPM packages now declare XKB/XCB build dependencies (`libxkbcommon-dev`, `libxkbcommon-x11-dev`, `libxcb1-dev`, `libxcb-render0-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`); README updated with the matching install instructions
- Debian package version bumped to `2026.05.03`

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
