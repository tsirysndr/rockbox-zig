# rockbox_ex

Idiomatic Elixir SDK for [Rockbox Zig](https://github.com/tsirysndr/rockbox-zig) — a fully typed
GraphQL client for `rockboxd` with real-time WebSocket subscriptions, a
plugin behaviour, and a builder DSL for smart playlists.

- **Pipe-friendly** — every API function takes the client as its first arg.
- **Builder-friendly** — smart-playlist rules and partial settings updates
  compose with `|>`.
- **Tagged tuples or bangs** — `name/N → {:ok, value} | {:error, exception}`,
  with a matching `name!/N` that raises.
- **Real-time events as messages** — `Rockbox.subscribe(:track_changed)` and
  receive `{:rockbox, :track_changed, %Rockbox.Track{}}`.
- **Plugins** — implement `Rockbox.Plugin` and install with
  `Rockbox.use_plugin/2`.

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
- [Real-time events](#real-time-events)
- [Plugins](#plugins)
- [Error handling](#error-handling)
- [Raw GraphQL queries](#raw-graphql-queries)

---

## Installation

```elixir
def deps do
  [
    {:rockbox_ex, "~> 0.1"}
  ]
end
```

`rockboxd` must be running and reachable. By default the SDK connects to
`http://localhost:6062/graphql`. Start rockboxd with:

```sh
rockbox start
```

---

## Quick start

```elixir
client = Rockbox.new()

# Optional: open the WebSocket so subscribers receive events
{:ok, _pid} = Rockbox.connect(client)

# What's playing right now?
case Rockbox.Playback.current_track(client) do
  {:ok, %Rockbox.Track{} = t} -> IO.puts("▶ #{t.title} — #{t.artist}")
  {:ok, nil}                  -> IO.puts("Nothing is playing.")
end

# Search the library
{:ok, results} = Rockbox.Library.search(client, "dark side")
album = List.first(results.albums)

# Play it shuffled
:ok = Rockbox.Playback.play_album(client, album.id, shuffle: true)

# React to track changes
:ok = Rockbox.subscribe(:track_changed)

receive do
  {:rockbox, :track_changed, track} ->
    IO.puts("Now: #{track.title}")
end

# Tear down when done
Rockbox.disconnect(client)
```

---

## Configuration

```elixir
# Defaults: localhost:6062
client = Rockbox.new()

# Custom host and port
client = Rockbox.new(host: "192.168.1.42", port: 6062)

# Fully custom URLs (useful behind a reverse proxy)
client = Rockbox.new(
  http_url: "https://music.home/graphql",
  ws_url:   "wss://music.home/graphql"
)
```

| Option      | Type                   | Default                          | Description                                         |
|-------------|------------------------|----------------------------------|-----------------------------------------------------|
| `:host`     | `String.t()`           | `"localhost"`                    | Hostname or IP of rockboxd                          |
| `:port`     | `non_neg_integer()`    | `6062`                           | GraphQL HTTP/WS port                                |
| `:http_url` | `String.t()`           | `http://{host}:{port}/graphql`   | Override the full HTTP URL                          |
| `:ws_url`   | `String.t()`           | `ws://{host}:{port}/graphql`     | Override the full WebSocket URL                     |
| `:headers`  | `[{String.t(), String.t()}]` | `[]`                       | Extra HTTP request headers                          |
| `:timeout`  | `non_neg_integer()`    | `15_000`                         | HTTP request timeout (ms)                           |

---

## API reference

Every function comes in two flavors:

- `name/N → {:ok, value} | {:error, exception}` — for `with`/`case` pipelines.
- `name!/N → value` — raises `Rockbox.Error` (or a subclass) on failure.

### Playback

```elixir
# Status — returns an atom: :stopped | :playing | :paused
{:ok, :playing} = Rockbox.Playback.status(client)

# Toggle
case Rockbox.Playback.status!(client) do
  :playing -> Rockbox.Playback.pause(client)
  _        -> Rockbox.Playback.resume(client)
end

# Transport
:ok = Rockbox.Playback.next(client)
:ok = Rockbox.Playback.previous(client)
:ok = Rockbox.Playback.stop(client)

# Seek to absolute position (ms)
:ok = Rockbox.Playback.seek(client, 90_000)

# Current / next track — returns nil when stopped
{:ok, %Rockbox.Track{title: t}} = Rockbox.Playback.current_track(client)
{:ok, _next}                    = Rockbox.Playback.next_track(client)

# Play helpers — single-call shortcuts
:ok = Rockbox.Playback.play_track(client, "/Music/foo.mp3")
:ok = Rockbox.Playback.play_album(client, "album-id", shuffle: true)
:ok = Rockbox.Playback.play_artist(client, "artist-id", shuffle: true)
:ok = Rockbox.Playback.play_playlist(client, "playlist-id")
:ok = Rockbox.Playback.play_directory(client, "/Music/Jazz", recurse: true, shuffle: true)
:ok = Rockbox.Playback.play_liked_tracks(client, shuffle: true)
:ok = Rockbox.Playback.play_all_tracks(client, shuffle: true)
```

`Rockbox.Track` exposes a couple of helpers:

```elixir
Rockbox.Track.format_length(track)   # "4:32"
Rockbox.Track.format_elapsed(track)  # "1:14"
Rockbox.Track.progress(track)        # 0.27  (0.0–1.0)
```

### Library

```elixir
# Albums
{:ok, albums} = Rockbox.Library.albums(client)
{:ok, album}  = Rockbox.Library.album(client, "album-id")    # full track list
{:ok, liked}  = Rockbox.Library.liked_albums(client)
:ok = Rockbox.Library.like_album(client, "album-id")
:ok = Rockbox.Library.unlike_album(client, "album-id")

# Artists
{:ok, artists} = Rockbox.Library.artists(client)
{:ok, artist}  = Rockbox.Library.artist(client, "artist-id")

# Tracks
{:ok, tracks} = Rockbox.Library.tracks(client)
{:ok, track}  = Rockbox.Library.track(client, "track-id")
{:ok, liked}  = Rockbox.Library.liked_tracks(client)
:ok = Rockbox.Library.like_track(client, "track-id")
:ok = Rockbox.Library.unlike_track(client, "track-id")

# Search across artists, albums, tracks, liked
{:ok, results} = Rockbox.Library.search(client, "radiohead")
results.artists       # [%Rockbox.Artist{}, ...]
results.albums        # [%Rockbox.Album{}, ...]
results.tracks        # [%Rockbox.Track{}, ...]
results.liked_tracks
results.liked_albums

# Trigger a full library rescan
:ok = Rockbox.Library.scan(client)
```

### Queue (live playlist)

The *queue* is the live playback list — what plays right now. For persistent
named collections see [Saved playlists](#saved-playlists).

```elixir
{:ok, queue} = Rockbox.Queue.current(client)
queue.amount       # total tracks
queue.index        # 0-based position of the currently playing track
queue.tracks       # [%Rockbox.Track{}, ...]

Rockbox.Playlist.current_track(queue)  # convenience helper

# Insertion: position is :next | :after_current | :last | :first
:ok = Rockbox.Queue.insert_tracks(client, ["/Music/a.mp3", "/Music/b.mp3"], :next)
:ok = Rockbox.Queue.insert_directory(client, "/Music/Ambient", :last)
:ok = Rockbox.Queue.insert_album(client, "album-id", :next)

# Other ops
:ok = Rockbox.Queue.remove_track(client, 2)
:ok = Rockbox.Queue.clear(client)
:ok = Rockbox.Queue.shuffle(client)
:ok = Rockbox.Queue.create(client, "Evening Mix", ["/a.mp3", "/b.mp3"])
:ok = Rockbox.Queue.resume(client)

# Pipe-friendly chaining with bang variants
client
|> tap(&Rockbox.Queue.clear!/1)
|> tap(&Rockbox.Queue.insert_tracks!(&1, ["/Music/a.mp3"], :last))
|> Rockbox.Queue.shuffle!()
```

### Saved playlists

```elixir
{:ok, lists} = Rockbox.SavedPlaylists.list(client)
{:ok, lists} = Rockbox.SavedPlaylists.list(client, "folder-id")

{:ok, pl}    = Rockbox.SavedPlaylists.get(client, "playlist-id")
{:ok, ids}   = Rockbox.SavedPlaylists.track_ids(client, "playlist-id")

# Create
{:ok, pl} =
  Rockbox.SavedPlaylists.create(client,
    name: "Late Night Jazz",
    description: "Quiet music for working",
    folder_id: "folder-id",        # optional
    track_ids: ["t1", "t2", "t3"]  # optional
  )

# Update / add / remove
:ok = Rockbox.SavedPlaylists.update(client, pl.id, name: "Late Night Jazz (v2)")
:ok = Rockbox.SavedPlaylists.add_tracks(client, pl.id, ["t4", "t5"])
:ok = Rockbox.SavedPlaylists.remove_track(client, pl.id, "t1")

# Play / delete
:ok = Rockbox.SavedPlaylists.play(client, pl.id)
:ok = Rockbox.SavedPlaylists.delete(client, pl.id)

# Folders
{:ok, folders} = Rockbox.SavedPlaylists.folders(client)
{:ok, folder}  = Rockbox.SavedPlaylists.create_folder(client, "Work")
:ok = Rockbox.SavedPlaylists.delete_folder(client, folder.id)
```

### Smart playlists

Use the `Rockbox.SmartPlaylist.Rules` builder — pipe-friendly, type-safe.

```elixir
alias Rockbox.SmartPlaylist.Rules

rules =
  Rules.all_of()
  |> Rules.where(:play_count, :gte, 10)
  |> Rules.where(:last_played, :within, "30d")
  |> Rules.sort(:play_count, :desc)
  |> Rules.limit(50)
  |> Rules.to_json()

{:ok, sp} =
  Rockbox.SmartPlaylists.create(client,
    name: "Most played (last 30d)",
    description: "Top 50 most-played tracks from the last month",
    rules: rules
  )

{:ok, ids} = Rockbox.SmartPlaylists.track_ids(client, sp.id)
:ok = Rockbox.SmartPlaylists.play(client, sp.id)
:ok = Rockbox.SmartPlaylists.delete(client, sp.id)

# OR groups
or_rules =
  Rules.any_of()
  |> Rules.where(:title, :contains, "Live")
  |> Rules.where(:title, :contains, "Acoustic")

# Mixed AND/OR via where_group/2
mixed =
  Rules.all_of()
  |> Rules.where(:play_count, :gt, 0)
  |> Rules.where_group(or_rules)
  |> Rules.to_json()
```

#### Listening stats

```elixir
{:ok, stats} = Rockbox.SmartPlaylists.track_stats(client, "track-id")

# Record events manually (e.g. from a scrobbler plugin)
:ok = Rockbox.SmartPlaylists.record_played(client, "track-id")
:ok = Rockbox.SmartPlaylists.record_skipped(client, "track-id")
```

### Sound

Volume is adjusted in firmware-defined steps. The number of steps per dB
varies by hardware target — always inspect `volume/1` for the range.

```elixir
{:ok, %Rockbox.Volume{volume: v, min: lo, max: hi}} = Rockbox.Sound.volume(client)

{:ok, new_value} = Rockbox.Sound.adjust(client, 3)   # +3 steps
{:ok, _}         = Rockbox.Sound.up(client)          # +1
{:ok, _}         = Rockbox.Sound.down(client)        # -1
```

### Settings

`update/2` accepts any subset of fields — only the ones you pass are written.

```elixir
{:ok, settings} = Rockbox.Settings.get(client)

# Toggle shuffle + repeat
:ok = Rockbox.Settings.update(client, shuffle: true, repeat_mode: 1)

# Equalizer
:ok =
  Rockbox.Settings.update(client,
    eq_enabled: true,
    eq_precut: -3,
    eq_band_settings: [
      %{cutoff: 60,    q: 7, gain:  3},
      %{cutoff: 200,   q: 7, gain:  0},
      %{cutoff: 4000,  q: 7, gain: -2}
    ]
  )

# Compressor
:ok =
  Rockbox.Settings.update(client,
    compressor_settings: %{
      threshold: -24, makeup_gain: 3, ratio: 2,
      knee: 0, release_time: 100, attack_time: 5
    }
  )

# Replaygain
:ok =
  Rockbox.Settings.update(client,
    replaygain_settings: %{noclip: true, type: 1, preamp: 0}
  )
```

### System

```elixir
{:ok, version} = Rockbox.System.version(client)
{:ok, status}  = Rockbox.System.status(client)

status.runtime          # seconds since boot
status.topruntime       # peak runtime
status.resume_index     # last queued position
```

### Browse (filesystem)

```elixir
{:ok, entries} = Rockbox.Browse.entries(client)                   # music_dir root
{:ok, entries} = Rockbox.Browse.entries(client, "/Music/Pink Floyd")

for e <- entries do
  icon = if Rockbox.Entry.directory?(e), do: "📁", else: "🎵"
  IO.puts("#{icon} #{e.name}")
end

{:ok, dirs}  = Rockbox.Browse.directories(client, "/Music")
{:ok, files} = Rockbox.Browse.files(client, "/Music/Pink Floyd/The Wall")
```

### Devices

```elixir
{:ok, devices} = Rockbox.Devices.list(client)
{:ok, device}  = Rockbox.Devices.get(client, "device-id")

# Connect — switches the active PCM output sink to this device
:ok = Rockbox.Devices.connect(client, "chromecast-id")
:ok = Rockbox.Devices.disconnect(client, "chromecast-id")
```

### Bluetooth

Linux only — backed by BlueZ. Calls return a `Rockbox.GraphQLError` on
non-Linux hosts.

```elixir
{:ok, devices} = Rockbox.Bluetooth.devices(client)
{:ok, found}   = Rockbox.Bluetooth.scan(client, 10)   # 10 second scan
:ok = Rockbox.Bluetooth.connect(client, "AA:BB:CC:DD:EE:FF")
:ok = Rockbox.Bluetooth.disconnect(client, "AA:BB:CC:DD:EE:FF")
```

---

## Real-time events

Open the WebSocket once with `Rockbox.connect/1`. The connection is supervised
and auto-reconnects with exponential backoff (capped at 30 s). Subscribers
receive plain Erlang messages, so they integrate with `receive` blocks and
`GenServer.handle_info/2`.

```elixir
client = Rockbox.new()
{:ok, _pid} = Rockbox.connect(client)

:ok = Rockbox.subscribe(:track_changed)
:ok = Rockbox.subscribe([:status_changed, :playlist_changed])  # multiple
:ok = Rockbox.subscribe(:all)                                  # catch-all

receive do
  {:rockbox, :track_changed, %Rockbox.Track{} = track} ->
    IO.puts("▶ #{track.title} — #{track.artist}")

  {:rockbox, :status_changed, status} ->
    IO.puts("status → #{status}")     # :stopped | :playing | :paused

  {:rockbox, :playlist_changed, %Rockbox.Playlist{} = queue} ->
    IO.puts("queue is now #{queue.amount} tracks")
end

Rockbox.unsubscribe(:track_changed)
Rockbox.disconnect(client)
```

### Event map

| Event                | Payload                                       |
|----------------------|-----------------------------------------------|
| `:track_changed`     | `%Rockbox.Track{}`                            |
| `:status_changed`    | `:stopped | :playing | :paused`               |
| `:playlist_changed`  | `%Rockbox.Playlist{}`                         |
| `:ws_open`           | `nil`                                         |
| `:ws_close`          | `nil`                                         |
| `:ws_error`          | `Exception.t()`                               |

Subscribers are auto-removed when their process exits — no manual cleanup
needed.

### Inside a GenServer

```elixir
defmodule MyApp.NowPlaying do
  use GenServer

  def start_link(client), do: GenServer.start_link(__MODULE__, client, name: __MODULE__)

  @impl true
  def init(client) do
    Rockbox.connect(client)
    Rockbox.subscribe([:track_changed, :status_changed])
    {:ok, %{client: client, track: nil, status: :stopped}}
  end

  @impl true
  def handle_info({:rockbox, :track_changed, track}, state),
    do: {:noreply, %{state | track: track}}

  def handle_info({:rockbox, :status_changed, status}, state),
    do: {:noreply, %{state | status: status}}
end
```

---

## Plugins

Plugins are the recommended way to bolt on cross-cutting features — scrobbling,
desktop notifications, analytics, sleep timers — without forking the SDK.

### Writing a plugin

```elixir
defmodule MyApp.LastFmScrobbler do
  @behaviour Rockbox.Plugin

  @impl true
  def name,        do: "lastfm-scrobbler"
  @impl true
  def version,     do: "1.0.0"
  @impl true
  def description, do: "Scrobble played tracks to Last.fm"

  @impl true
  def install(ctx) do
    {:ok, pid} = MyApp.LastFmScrobbler.Worker.start_link(ctx.client)
    {:ok, %{worker: pid}}
  end

  @impl true
  def uninstall(%{worker: pid}) do
    if Process.alive?(pid), do: GenServer.stop(pid)
    :ok
  end
end

defmodule MyApp.LastFmScrobbler.Worker do
  use GenServer

  def start_link(client), do: GenServer.start_link(__MODULE__, client)

  @impl true
  def init(client) do
    Rockbox.Events.subscribe(:track_changed)
    {:ok, %{client: client, current: nil, started_at: 0}}
  end

  @impl true
  def handle_info({:rockbox, :track_changed, track}, state) do
    now = System.monotonic_time(:millisecond)

    # Submit the previous track if it played for more than 30 s
    if state.current && now - state.started_at > 30_000 do
      submit_scrobble(state.current)
    end

    {:noreply, %{state | current: track, started_at: now}}
  end

  defp submit_scrobble(_track), do: :ok  # talk to the Last.fm API here
end
```

### Installing

```elixir
client = Rockbox.new()
{:ok, _} = Rockbox.connect(client)

:ok = Rockbox.use_plugin(client, MyApp.LastFmScrobbler)

# Inspect what's installed
for entry <- Rockbox.installed_plugins() do
  IO.puts("#{entry.module.name()} v#{entry.module.version()}")
end

:ok = Rockbox.unuse_plugin("lastfm-scrobbler")    # by name
:ok = Rockbox.unuse_plugin(MyApp.LastFmScrobbler) # or by module
```

The `install/1` callback receives `%{client: client}`. Return `{:ok, state}`;
the state is passed back to `uninstall/1` so resources can be cleaned up.

---

## Error handling

```elixir
case Rockbox.Playback.play(client) do
  :ok ->
    :ok

  {:error, %Rockbox.NetworkError{} = e} ->
    Logger.error("rockboxd unreachable: #{Exception.message(e)}")

  {:error, %Rockbox.GraphQLError{errors: errors}} ->
    for %{message: msg} <- errors, do: Logger.error("graphql: #{msg}")

  {:error, %Rockbox.Error{} = e} ->
    Logger.error("rockbox: #{Exception.message(e)}")
end

# …or use the bang variant inside a try/rescue
try do
  Rockbox.Playback.play!(client)
rescue
  e in Rockbox.NetworkError -> Logger.error("offline: #{e.message}")
  e in Rockbox.GraphQLError -> Logger.error("server: #{e.message}")
end
```

| Exception                 | When raised                                              |
|---------------------------|----------------------------------------------------------|
| `Rockbox.NetworkError`    | HTTP request fails or returns a non-2xx status           |
| `Rockbox.GraphQLError`    | Server returns `{ "errors": [...] }` in the response body |
| `Rockbox.Error`           | Base exception — rescue this to catch any SDK failure    |

---

## Raw GraphQL queries

For operations not yet covered by a dedicated function, drop down to
`Rockbox.query/3`. Variables can be a map or keyword list — snake_case keys
are converted to camelCase before being sent.

```elixir
{:ok, %{"rockboxVersion" => v}} =
  Rockbox.query(client, "query { rockboxVersion }")

{:ok, %{"album" => album}} =
  Rockbox.query(
    client,
    "query Album($id: String!) { album(id: $id) { id title artist year } }",
    id: "abc-123"
  )

# Mutation
:ok = Rockbox.query(client, "mutation Seek($t: Int!) { fastForwardRewind(newTime: $t) }", t: 120_000) |> elem(0) == :ok
```

The GraphiQL explorer is available at `http://localhost:6062/graphiql` while
rockboxd is running.

---

## Module map

| Domain                | Module                          |
|-----------------------|---------------------------------|
| Client constructor    | `Rockbox`, `Rockbox.Client`     |
| Transport controls    | `Rockbox.Playback`              |
| Library / search      | `Rockbox.Library`               |
| Live queue            | `Rockbox.Queue`                 |
| Saved playlists       | `Rockbox.SavedPlaylists`        |
| Smart playlists       | `Rockbox.SmartPlaylists`        |
| Smart-playlist rules  | `Rockbox.SmartPlaylist.Rules`   |
| Volume                | `Rockbox.Sound`                 |
| Settings              | `Rockbox.Settings`              |
| System info           | `Rockbox.System`                |
| Filesystem browser    | `Rockbox.Browse`                |
| Output devices        | `Rockbox.Devices`               |
| Bluetooth             | `Rockbox.Bluetooth`             |
| Real-time events      | `Rockbox.Events`                |
| Plugin behaviour      | `Rockbox.Plugin`, `Rockbox.Plugins` |
| Errors                | `Rockbox.Error`, `Rockbox.NetworkError`, `Rockbox.GraphQLError` |

---

## Development

```sh
mix deps.get
mix test
mix docs    # generates HTML docs in doc/
```

Examples live in `examples/`. Start `rockboxd`, then:

```sh
mix run examples/01_basic_playback.exs
mix run --no-halt examples/02_now_playing.exs
```

---

## License

MIT License. See [LICENSE](./LICENSE) for details.
