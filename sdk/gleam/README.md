# rockbox

[![Package Version](https://img.shields.io/hexpm/v/rockbox)](https://hex.pm/packages/rockbox)
[![Hex Docs](https://img.shields.io/badge/hex-docs-ffaff3)](https://hexdocs.pm/rockbox/)

Gleam SDK for [Rockbox Zig](https://github.com/tsirysndr/rockbox-zig) — a
typed, pipe-friendly client for the `rockboxd` GraphQL API.

- **Pipe-friendly** — every API function takes the client as its first arg.
- **Builder-friendly** — smart-playlist rules and partial settings updates
  compose with `|>`.
- **Tagged results** — every call returns `Result(value, rockbox/error.Error)`,
  so `case` and `use` flows stay flat.
- **Type-safe rules DSL** — compose smart-playlist rules with
  `rockbox/smart_playlists/rules` instead of hand-written JSON.

---

## Table of contents

- [Installation](#installation)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [API reference](#api-reference)
  - [Playback](#playback)
  - [Library](#library)
  - [Queue (live playlist)](#queue-live-playlist)
  - [Saved playlists](#saved-playlists)
  - [Smart playlists](#smart-playlists)
  - [Sound](#sound)
  - [Settings](#settings)
  - [System](#system)
  - [Browse (filesystem)](#browse-filesystem)
  - [Devices](#devices)
  - [Bluetooth](#bluetooth)
- [Error handling](#error-handling)
- [Raw GraphQL queries](#raw-graphql-queries)
- [Module map](#module-map)

---

## Installation

```sh
gleam add rockbox
```

`rockboxd` must be running and reachable. By default the SDK connects to
`http://localhost:6062/graphql`. Start rockboxd with:

```sh
rockbox start
```

---

## Quick start

```gleam
import gleam/io
import gleam/list
import gleam/option.{None, Some}
import rockbox
import rockbox/library
import rockbox/playback

pub fn main() {
  let client = rockbox.default_client()

  // What's playing right now?
  case playback.current_track(client) {
    Ok(Some(track)) -> io.println("▶ " <> track.title <> " — " <> track.artist)
    Ok(None) -> io.println("Nothing is playing.")
    Error(_) -> io.println("Could not reach rockboxd.")
  }

  // Search the library and play the first hit
  let assert Ok(results) = library.search(client, "dark side")
  case list.first(results.albums) {
    Ok(album) -> {
      let _ =
        playback.play_album(
          client,
          album.id,
          playback.play_options() |> playback.with_shuffle(True),
        )
      Nil
    }
    Error(_) -> Nil
  }
}
```

---

## Configuration

The `Builder` pattern lets you override defaults one field at a time:

```gleam
// Defaults: localhost:6062
let client = rockbox.default_client()

// Custom host and port
let client = rockbox.at(host: "192.168.1.42", port: 6062)

// Fully custom URL (e.g. behind a reverse proxy with TLS)
let client =
  rockbox.new()
  |> rockbox.url("https://music.home/graphql")
  |> rockbox.connect
```

| Setter           | Default               | Description                            |
|------------------|-----------------------|----------------------------------------|
| `host(_, value)` | `"localhost"`         | Hostname or IP of rockboxd             |
| `port(_, value)` | `6062`                | GraphQL HTTP port                      |
| `url(_, value)`  | derived from host/port | Override the full HTTP URL (wins over `host` / `port`) |

Use `rockbox.http_url(client)` to read back the resolved URL — handy for
tests and diagnostics.

---

## API reference

Every function returns `Result(value, rockbox/error.Error)`. Pattern-match
or use `let assert Ok(x) = …` in scripts where a failure should crash.

### Playback

```gleam
import rockbox/playback
import rockbox/types

// Status — typed: Stopped | Playing | Paused | UnknownStatus(Int)
let assert Ok(status) = playback.status(client)

// Toggle
case status {
  types.Playing -> { let _ = playback.pause(client) }
  _ -> { let _ = playback.resume(client) }
}

// Transport
let _ = playback.next(client)
let _ = playback.previous(client)
let _ = playback.stop(client)

// Seek to absolute position (ms)
let _ = playback.seek(client, 90_000)

// Current / next track — Ok(Some(track)) when present, Ok(None) when stopped
let assert Ok(now) = playback.current_track(client)
let assert Ok(next) = playback.next_track(client)
```

#### Play helpers

`PlayOptions` is a small builder for the optional `shuffle` / `position`
knobs accepted by every `play_*` shortcut:

```gleam
let opts =
  playback.play_options()
  |> playback.with_shuffle(True)
  |> playback.with_position(2)

let _ = playback.play_track(client, "/Music/foo.mp3")
let _ = playback.play_album(client, "album-id", opts)
let _ = playback.play_artist(client, "artist-id", opts)
let _ = playback.play_playlist(client, "playlist-id", opts)
let _ = playback.play_directory(client, "/Music/Jazz", True, opts)
let _ = playback.play_liked_tracks(client, opts)
let _ = playback.play_all_tracks(client, opts)
```

### Library

```gleam
import rockbox/library

// Albums
let assert Ok(albums) = library.albums(client)
let assert Ok(album) = library.album(client, "album-id")    // includes tracks
let assert Ok(liked) = library.liked_albums(client)
let _ = library.like_album(client, "album-id")
let _ = library.unlike_album(client, "album-id")

// Artists
let assert Ok(artists) = library.artists(client)
let assert Ok(artist) = library.artist(client, "artist-id")

// Tracks
let assert Ok(tracks) = library.tracks(client)
let assert Ok(track) = library.track(client, "track-id")
let assert Ok(liked) = library.liked_tracks(client)
let _ = library.like_track(client, "track-id")
let _ = library.unlike_track(client, "track-id")

// Search across artists, albums, tracks, liked
let assert Ok(results) = library.search(client, "radiohead")
results.artists       // List(Artist)
results.albums        // List(Album)
results.tracks        // List(Track)
results.liked_tracks
results.liked_albums

// Trigger a full library rescan
let _ = library.scan(client)
```

### Queue (live playlist)

The live queue lives in `rockbox/playlist`. For persistent named
collections see [Saved playlists](#saved-playlists).

```gleam
import gleam/option.{None}
import rockbox/playlist
import rockbox/types

let assert Ok(queue) = playlist.current(client)
queue.amount       // total tracks
queue.index        // 0-based position of the currently playing track
queue.tracks       // List(Track)

// Insertion: position is types.Next | types.AfterCurrent | types.Last | types.First
let _ =
  playlist.insert_tracks(
    client,
    ["/Music/a.mp3", "/Music/b.mp3"],
    types.Next,
    None,
  )
let _ =
  playlist.insert_directory(client, "/Music/Ambient", types.Last, None)
let _ = playlist.insert_album(client, "album-id", types.Next)

// Other ops
let _ = playlist.remove_track(client, 2)
let _ = playlist.clear(client)
let _ = playlist.shuffle(client)
let _ = playlist.create(client, "Evening Mix", ["/a.mp3", "/b.mp3"])
let _ = playlist.resume(client)

// Start from a specific position
let opts =
  playlist.start_options()
  |> playlist.at_index(3)
  |> playlist.at_elapsed(0)
let _ = playlist.start(client, opts)
```

### Saved playlists

```gleam
import gleam/option.{None, Some}
import rockbox/saved_playlists

let assert Ok(lists) = saved_playlists.list(client, None)
let assert Ok(scoped) = saved_playlists.list(client, Some("folder-id"))

let assert Ok(pl) = saved_playlists.get(client, "playlist-id")
let assert Ok(ids) = saved_playlists.track_ids(client, "playlist-id")

// Create
let input =
  saved_playlists.new("Late Night Jazz")
  |> saved_playlists.with_description("Quiet music for working")
  |> saved_playlists.with_folder("folder-id")
  |> saved_playlists.with_tracks(["t1", "t2", "t3"])

let assert Ok(pl) = saved_playlists.create(client, input)

// Update / add / remove
let patch =
  saved_playlists.update("Late Night Jazz (v2)")
  |> saved_playlists.update_description("Updated cover")

let _ = saved_playlists.save(client, pl.id, patch)
let _ = saved_playlists.add_tracks(client, pl.id, ["t4", "t5"])
let _ = saved_playlists.remove_track(client, pl.id, "t1")

// Play / delete
let _ = saved_playlists.play(client, pl.id)
let _ = saved_playlists.delete(client, pl.id)

// Folders
let assert Ok(folders) = saved_playlists.folders(client)
let assert Ok(folder) = saved_playlists.create_folder(client, "Work")
let _ = saved_playlists.delete_folder(client, folder.id)
```

### Smart playlists

Compose rules with the type-safe `rockbox/smart_playlists/rules` builder
instead of hand-writing JSON.

```gleam
import rockbox/smart_playlists
import rockbox/smart_playlists/rules

let r =
  rules.all_of()
  |> rules.where("play_count", rules.Gte, rules.int(10))
  |> rules.where("last_played", rules.Within, rules.string("30d"))
  |> rules.sort("play_count", rules.Desc)
  |> rules.limit(50)

let input =
  smart_playlists.new("Most played (last 30d)", rules.to_string(r))
  |> smart_playlists.with_description("Top 50 most-played tracks from the last month")

let assert Ok(sp) = smart_playlists.create(client, input)

let assert Ok(ids) = smart_playlists.track_ids(client, sp.id)
let _ = smart_playlists.play(client, sp.id)
let _ = smart_playlists.delete(client, sp.id)
```

#### Operators

| Variant     | Meaning                                |
|-------------|----------------------------------------|
| `Eq`        | equals                                 |
| `Neq`       | not equals                             |
| `Gt`        | greater than                           |
| `Gte`       | greater than or equal                  |
| `Lt`        | less than                              |
| `Lte`       | less than or equal                     |
| `Contains`  | substring match                        |
| `Within`    | duration window (e.g. `"30d"`, `"7d"`) |

#### OR groups and nesting

```gleam
let either =
  rules.any_of()
  |> rules.where("title", rules.Contains, rules.string("Live"))
  |> rules.where("title", rules.Contains, rules.string("Acoustic"))

let mixed =
  rules.all_of()
  |> rules.where("play_count", rules.Gt, rules.int(0))
  |> rules.where_group(either)
```

#### Listening stats

```gleam
let assert Ok(stats) = smart_playlists.track_stats(client, "track-id")

// Record events manually (e.g. from a scrobbler)
let _ = smart_playlists.record_played(client, "track-id")
let _ = smart_playlists.record_skipped(client, "track-id")
```

### Sound

Volume is adjusted in firmware-defined steps. The number of steps per dB
varies by hardware target — always inspect `get_volume/1` for the range.

```gleam
import rockbox/sound

let assert Ok(vol) = sound.get_volume(client)
vol.volume      // current value
vol.min         // lower bound
vol.max         // upper bound

let assert Ok(new_value) = sound.adjust_volume(client, 3)   // +3 steps
let assert Ok(_) = sound.volume_up(client)                  // +1
let assert Ok(_) = sound.volume_down(client)                // -1
```

### Settings

`save/2` accepts any subset of fields — only the ones you set are written.

```gleam
import rockbox/settings
import rockbox/types.{
  CompressorSettings, EqBandSetting, ReplaygainSettings,
}

let assert Ok(current) = settings.get(client)

// Toggle shuffle + repeat
let patch =
  settings.patch()
  |> settings.set_shuffle(True)
  |> settings.set_repeat_mode(1)
let _ = settings.save(client, patch)

// Equalizer
let bands = [
  EqBandSetting(cutoff: 60, q: 7, gain: 3),
  EqBandSetting(cutoff: 200, q: 7, gain: 0),
  EqBandSetting(cutoff: 4000, q: 7, gain: -2),
]
let patch =
  settings.patch()
  |> settings.set_eq_enabled(True)
  |> settings.set_eq_precut(-3)
  |> settings.set_eq_bands(bands)
let _ = settings.save(client, patch)

// Compressor
let patch =
  settings.patch()
  |> settings.set_compressor(CompressorSettings(
    threshold: -24,
    makeup_gain: 3,
    ratio: 2,
    knee: 0,
    release_time: 100,
    attack_time: 5,
  ))
let _ = settings.save(client, patch)

// Replaygain
let patch =
  settings.patch()
  |> settings.set_replaygain(ReplaygainSettings(
    noclip: True, type_: 1, preamp: 0,
  ))
let _ = settings.save(client, patch)
```

### System

```gleam
import rockbox/system

let assert Ok(version) = system.version(client)
let assert Ok(status) = system.status(client)

status.runtime          // seconds since boot
status.topruntime       // peak runtime
status.resume_index     // last queued position
```

### Browse (filesystem)

```gleam
import gleam/option.{None, Some}
import rockbox/browse
import rockbox/types

let assert Ok(entries) = browse.entries(client, None)                   // music_dir root
let assert Ok(entries) = browse.entries(client, Some("/Music/Pink Floyd"))

list.each(entries, fn(e) {
  let icon = case types.is_directory(e) {
    True -> "[dir] "
    False -> "      "
  }
  io.println(icon <> e.name)
})

let assert Ok(dirs) = browse.directories(client, Some("/Music"))
let assert Ok(files) = browse.files(client, Some("/Music/Pink Floyd/The Wall"))
```

### Devices

```gleam
import rockbox/devices

let assert Ok(devices) = devices.list(client)
let assert Ok(device) = devices.get(client, "device-id")

// Connect — switches the active PCM output sink to this device
let _ = devices.connect(client, "chromecast-id")
let _ = devices.disconnect(client, "chromecast-id")
```

### Bluetooth

Linux only — backed by BlueZ. Calls return a `GraphQLError` on non-Linux
hosts.

```gleam
import gleam/option.{None, Some}
import rockbox/bluetooth

let assert Ok(devices) = bluetooth.devices(client)
let assert Ok(found) = bluetooth.scan(client, Some(10))   // 10 second scan
let _ = bluetooth.connect(client, "AA:BB:CC:DD:EE:FF")
let _ = bluetooth.disconnect(client, "AA:BB:CC:DD:EE:FF")
```

---

## Error handling

```gleam
import rockbox/error
import rockbox/playback

case playback.current_track(client) {
  Ok(track) -> echo track
  Error(error.NetworkError(reason)) -> io.println("offline: " <> reason)
  Error(error.HttpError(status, _)) -> io.println("http " <> int.to_string(status))
  Error(error.GraphQLError(messages)) ->
    list.each(messages, fn(m) { io.println("server: " <> m) })
  Error(error.DecodeError(reason)) -> io.println("decode: " <> reason)
}
```

| Variant         | When raised                                              |
|-----------------|----------------------------------------------------------|
| `NetworkError`  | DNS, refused connection, TLS, etc.                       |
| `HttpError`     | Server returned a non-2xx HTTP response.                 |
| `GraphQLError`  | Server returned a populated `errors` array.              |
| `DecodeError`   | Response body could not be decoded into the expected shape. |

---

## Raw GraphQL queries

For operations not yet covered by a dedicated function, drop down to
`rockbox.query/4` (or `rockbox.execute/3` for fire-and-forget mutations)
and supply your own decoder.

```gleam
import gleam/dynamic/decode
import gleam/json

let version_decoder = {
  use v <- decode.field("rockboxVersion", decode.string)
  decode.success(v)
}

let assert Ok(version) =
  rockbox.query(
    client,
    "query Version { rockboxVersion }",
    json.object([]),
    version_decoder,
  )

// Mutation — use execute when you don't care about the response body
let _ =
  rockbox.execute(
    client,
    "mutation Seek($t: Int!) { fastForwardRewind(newTime: $t) }",
    json.object([#("t", json.int(120_000))]),
  )
```

The GraphiQL explorer is available at `http://localhost:6062/graphiql`
while rockboxd is running.

---

## Module map

| Domain                | Module                                  |
|-----------------------|-----------------------------------------|
| Client constructor    | `rockbox`                               |
| Transport controls    | `rockbox/playback`                      |
| Library / search      | `rockbox/library`                       |
| Live queue            | `rockbox/playlist`                      |
| Saved playlists       | `rockbox/saved_playlists`               |
| Smart playlists       | `rockbox/smart_playlists`               |
| Smart-playlist rules  | `rockbox/smart_playlists/rules`         |
| Volume                | `rockbox/sound`                         |
| Settings              | `rockbox/settings`                      |
| System info           | `rockbox/system`                        |
| Filesystem browser    | `rockbox/browse`                        |
| Output devices        | `rockbox/devices`                       |
| Bluetooth             | `rockbox/bluetooth`                     |
| Domain types          | `rockbox/types`                         |
| Errors                | `rockbox/error`                         |

---

## Development

```sh
gleam test    # run the test suite
gleam docs build
```

Runnable examples live in `examples/`. Start `rockboxd`, then:

```sh
cd examples
gleam run -m example_01_basic_playback
gleam run -m example_06_smart_playlist
```

See [`examples/README.md`](./examples/README.md) for the full list.

Further documentation is on [HexDocs](https://hexdocs.pm/rockbox).

---

## License

MIT License. See [LICENSE](./LICENSE) for details.
