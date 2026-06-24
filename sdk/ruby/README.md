# rockbox

[![Gem Version](https://img.shields.io/gem/v/rockbox?style=flat-square&logo=rubygems&logoColor=white&color=E9573F)](https://rubygems.org/gems/rockbox)
[![Ruby](https://img.shields.io/badge/ruby-%3E%3D%203.0-CC342D?style=flat-square&logo=ruby&logoColor=white)](https://www.ruby-lang.org/)
[![Downloads](https://img.shields.io/gem/dt/rockbox?style=flat-square&logo=rubygems&logoColor=white&color=blue)](https://rubygems.org/gems/rockbox)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
[![GraphQL](https://img.shields.io/badge/GraphQL-client-E10098?style=flat-square&logo=graphql&logoColor=white)](https://graphql.org/)
[![WebSocket](https://img.shields.io/badge/WebSocket-realtime-4353FF?style=flat-square&logo=socketdotio&logoColor=white)](https://github.com/enhancv/websocket-client-simple)
[![Plugins](https://img.shields.io/badge/plugins-extensible-8A2BE2?style=flat-square&logo=rubygems&logoColor=white)](#plugin-system)
[![GitHub](https://img.shields.io/badge/github-rockboxd-181717?style=flat-square&logo=github&logoColor=white)](https://github.com/tsirysndr/rockboxd)

Ruby SDK for [Rockbox Daemon](https://github.com/tsirysndr/rockboxd) — a builder-friendly, block-friendly GraphQL client with real-time event subscriptions and a plugin system.

```ruby
require "rockbox"

client = Rockbox::Client.build do |c|
  c.host = "localhost"
  c.port = 6062
end

client.on(:track_changed) { |t| puts "▶ #{t.title} — #{t.artist}" }
client.connect

results = client.library.search("dark side")
client.playback.play_album(results.albums.first.id, shuffle: true)
```

---

## Table of contents

- [Installation](#installation)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [API reference](#api-reference)
  - [Playback](#playback)
  - [Library](#library)
  - [Playlist (queue)](#playlist-queue)
  - [Saved playlists](#saved-playlists)
  - [Smart playlists](#smart-playlists)
  - [Sound](#sound)
  - [Settings](#settings)
  - [System](#system)
  - [Browse (filesystem)](#browse-filesystem)
  - [Devices](#devices)
  - [Bluetooth](#bluetooth)
- [Real-time events](#real-time-events)
- [Plugin system](#plugin-system)
- [Error handling](#error-handling)
- [Raw GraphQL queries](#raw-graphql-queries)

---

## Installation

```sh
gem install rockbox
```

Or with Bundler:

```ruby
# Gemfile
gem "rockbox"
```

`rockboxd` must be running and reachable. The SDK targets **Ruby 3.0+** and connects to `http://localhost:6062/graphql` by default.

---

## Quick start

```ruby
require "rockbox"

client = Rockbox::Client.new

# Optional — start WebSocket subscriptions for real-time events.
client.connect

# What's playing?
if (track = client.playback.current_track)
  puts "Now playing: #{track.title} — #{track.artist}"
end

# Search the library.
results = client.library.search("dark side")
puts "Found #{results.albums.size} albums and #{results.tracks.size} tracks"

# Play an album with shuffle.
client.playback.play_album(results.albums.first.id, shuffle: true)

# React to track changes.
client.on(:track_changed) do |track|
  puts "▶ #{track.title} by #{track.artist}"
end

# Tear down when done.
client.disconnect
```

---

## Configuration

Three equivalent ways to configure the client. Pick the one that fits your style.

```ruby
# 1. Defaults — localhost:6062.
client = Rockbox::Client.new

# 2. Keyword arguments.
client = Rockbox::Client.new(host: "192.168.1.42", port: 6062)

# 3. Builder block (great for application initializers).
client = Rockbox::Client.build do |c|
  c.host = "192.168.1.42"
  c.port = 6062
end

# 4. Fully custom URLs (useful behind a reverse proxy).
client = Rockbox::Client.new(
  http_url: "https://music.home/graphql",
  ws_url:   "wss://music.home/graphql"
)

# Top-level shorthand.
client = Rockbox.new(host: "localhost")
```

| Option         | Type    | Default                        | Description                     |
| -------------- | ------- | ------------------------------ | ------------------------------- |
| `host`         | String  | `"localhost"`                  | Hostname or IP of rockboxd      |
| `port`         | Integer | `6062`                         | GraphQL port                    |
| `http_url`     | String  | `http://{host}:{port}/graphql` | Override the full HTTP URL      |
| `ws_url`       | String  | `ws://{host}:{port}/graphql`   | Override the full WebSocket URL |
| `open_timeout` | Integer | `5`                            | HTTP connect timeout (seconds)  |
| `read_timeout` | Integer | `30`                           | HTTP read timeout (seconds)     |

---

## API reference

The client exposes a domain namespace per concern (Mopidy-style). Every method returns idiomatic Ruby data — `Struct` instances with `snake_case` accessors.

### Playback

```ruby
client.playback
```

```ruby
# Status
client.playback.status         # => Integer (Rockbox::PlaybackStatus::PLAYING, etc.)
client.playback.status_name    # => :playing | :paused | :stopped | :unknown

# Current/next track
track = client.playback.current_track  # => Rockbox::Track
client.playback.next_track
client.playback.file_position

# Transport controls
client.playback.play(elapsed: 0, offset: 0)
client.playback.pause
client.playback.resume
client.playback.next!
client.playback.previous!
client.playback.seek(60_000)            # ms
client.playback.stop
client.playback.flush_and_reload

# One-shot play helpers
client.playback.play_track("/Music/song.mp3")
client.playback.play_album(album_id, shuffle: true)
client.playback.play_artist(artist_id)
client.playback.play_playlist(playlist_id, shuffle: true, position: 0)
client.playback.play_directory("/Music/Pink Floyd", recurse: true)
client.playback.play_liked_tracks(shuffle: true)
client.playback.play_all_tracks
```

### Library

```ruby
# Albums
client.library.albums                 # => Array<Rockbox::Album>
client.library.album(id)              # => Rockbox::Album | nil
client.library.liked_albums
client.library.like_album(id)
client.library.unlike_album(id)

# Artists
client.library.artists
client.library.artist(id)

# Tracks
client.library.tracks
client.library.track(id)
client.library.liked_tracks
client.library.like_track(id)
client.library.unlike_track(id)

# Search
results = client.library.search("daft punk")
results.artists      # => Array<Rockbox::Artist>
results.albums       # => Array<Rockbox::Album>
results.tracks       # => Array<Rockbox::Track>
results.liked_tracks
results.liked_albums

# Library scan
client.library.scan
```

### Playlist (queue)

```ruby
playlist = client.playlist.current
playlist.amount       # 42
playlist.index        # currently playing index
playlist.tracks       # Array<Rockbox::Track>

client.playlist.amount  # convenience for playlist.current.amount

# Inserts (paths or track IDs)
client.playlist.insert_tracks(["/Music/a.mp3", "/Music/b.mp3"],
                              position: Rockbox::InsertPosition::NEXT)
client.playlist.insert_directory("/Music/Pink Floyd", position: Rockbox::InsertPosition::LAST)
client.playlist.insert_album(album_id)

# Mutations
client.playlist.remove_track(3)
client.playlist.clear
client.playlist.shuffle

# Create + start a temporary playlist
client.playlist.create("Tonight", ["/Music/a.mp3", "/Music/b.mp3"])
client.playlist.start(start_index: 0)
client.playlist.resume
```

`Rockbox::InsertPosition` constants: `NEXT`, `AFTER_CURRENT`, `LAST`, `FIRST`.

### Saved playlists

```ruby
client.saved_playlists.list                     # => Array<Rockbox::SavedPlaylist>
client.saved_playlists.list(folder_id: "f_1")
client.saved_playlists.get("pl_42")
client.saved_playlists.track_ids("pl_42")

# Builder block (any field is optional)
playlist = client.saved_playlists.create(name: "Late nights") do |p|
  p.description = "After-dark vibes"
  p.image       = "https://…/cover.png"
  p.track_ids   = ["abc", "def"]
end

# Or pass kwargs directly
client.saved_playlists.update("pl_42", name: "Renamed", description: "…")
client.saved_playlists.add_tracks("pl_42", ["abc", "def"])
client.saved_playlists.remove_track("pl_42", "abc")
client.saved_playlists.delete("pl_42")
client.saved_playlists.play("pl_42")

# Folders
client.saved_playlists.folders
client.saved_playlists.create_folder("Workout")
client.saved_playlists.delete_folder("f_1")
```

### Smart playlists

```ruby
client.smart_playlists.list
client.smart_playlists.get(id)
client.smart_playlists.track_ids(id)
client.smart_playlists.create(name: "Heavy hitters", rules: rules_json)
client.smart_playlists.update(id, name: "…", rules: new_rules)
client.smart_playlists.delete(id)
client.smart_playlists.play(id)

# Listening stats
client.smart_playlists.track_stats(track_id)
client.smart_playlists.record_played(track_id)
client.smart_playlists.record_skipped(track_id)
```

### Sound

```ruby
info = client.sound.volume        # => Rockbox::VolumeInfo(volume:, min:, max:)
client.sound.adjust(3)            # +3 steps
client.sound.up                   # +1 step
client.sound.down                 # -1 step
```

### Settings

```ruby
settings = client.settings.get    # => Rockbox::UserSettings (Struct)
settings.volume       # -20
settings.shuffle      # false

# Save with a builder block — only the fields you set are sent.
client.settings.save do |s|
  s.volume  = -10
  s.bass    = 4
  s.shuffle = true
end

# Or with a plain Hash.
client.settings.save(volume: -10, shuffle: true)
```

### System

```ruby
client.system.version             # "Rockbox v…"
client.system.status              # Rockbox::SystemStatus
```

### Browse (filesystem)

```ruby
client.browse.entries("/Music")           # => Array<Rockbox::Entry>
client.browse.directories("/Music")        # only directories
client.browse.files("/Music/Pink Floyd")   # only files

entry = client.browse.entries.first
Rockbox.directory?(entry)   # => true/false
```

### Devices

```ruby
client.devices.list             # => Array<Rockbox::Device>
client.devices.get(id)
client.devices.connect(id)
client.devices.disconnect(id)
```

### Bluetooth

> Linux only. macOS/Windows builds will return errors.

```ruby
client.bluetooth.devices
client.bluetooth.scan(timeout: 5)
client.bluetooth.connect("AA:BB:CC:DD:EE:FF")
client.bluetooth.disconnect("AA:BB:CC:DD:EE:FF")
```

---

## Real-time events

Call `#connect` to open the WebSocket and start receiving events. The client speaks the GraphQL `graphql-transport-ws` subprotocol.

```ruby
client = Rockbox::Client.new
client.connect

client.on(:track_changed)    { |track|    puts "▶ #{track.title}" }
client.on(:status_changed)   { |status|   puts Rockbox::PlaybackStatus.name(status) }
client.on(:playlist_changed) { |playlist| puts "queue: #{playlist.amount} tracks" }
client.on(:ws_open)          { puts "connected" }
client.on(:ws_close)         { puts "disconnected" }
client.on(:ws_error)         { |err| warn err.message }

# Once-only listener
client.once(:track_changed) { |t| puts "first track: #{t.title}" }

# Remove a specific listener with the same Proc
listener = ->(t) { puts t.title }
client.on(:track_changed, &listener)
client.off(:track_changed, listener)

# Remove all listeners for an event (or all events)
client.remove_all_listeners(:track_changed)
client.remove_all_listeners

# Tear down
client.disconnect
```

| Event               | Payload                               |
| ------------------- | ------------------------------------- |
| `:track_changed`    | `Rockbox::Track`                      |
| `:status_changed`   | `Integer` (`Rockbox::PlaybackStatus`) |
| `:playlist_changed` | `Rockbox::Playlist`                   |
| `:ws_open`          | `nil`                                 |
| `:ws_close`         | `nil`                                 |
| `:ws_error`         | `Exception`                           |

---

## Plugin system

Plugins are duck-typed objects with `#name`, `#version`, and `#install(context)`. Inherit from `Rockbox::Plugin` for sane defaults.

```ruby
class ConsoleScrobbler < Rockbox::Plugin
  def name;        "console-scrobbler" end
  def version;     "0.1.0"             end
  def description; "Logs every track change" end

  def install(ctx)
    ctx.events.on(:track_changed) do |track|
      puts "♪ #{track.artist} — #{track.title}"
    end
  end

  def uninstall
    # cleanup
  end
end

client.use(ConsoleScrobbler.new)
client.installed_plugins      # => [<ConsoleScrobbler ...>]
client.unuse("console-scrobbler")
```

The `PluginContext` exposes:

- `ctx.query.call(gql, variables = nil)` — issue raw GraphQL operations
- `ctx.events` — the same `Rockbox::EventEmitter` used by the client

---

## Error handling

```ruby
begin
  client.library.album("does-not-exist")
rescue Rockbox::GraphQLError => e
  e.errors.each { |err| warn err[:message] }
rescue Rockbox::NetworkError => e
  warn "rockboxd unreachable: #{e.message}"
end
```

| Class                   | Raised when…                                 |
| ----------------------- | -------------------------------------------- |
| `Rockbox::Error`        | Base class for every SDK error.              |
| `Rockbox::NetworkError` | rockboxd is unreachable / non-2xx response.  |
| `Rockbox::GraphQLError` | rockboxd returns a GraphQL `errors` payload. |

---

## Raw GraphQL queries

For operations the SDK doesn't yet wrap, use `#query` directly. Variables are camelized on the way out, response keys are snakeized on the way in.

```ruby
data = client.query(
  "query LikedSongs { likedTracks { id title } }"
)
data[:liked_tracks].each { |t| puts t[:title] }

data = client.query(<<~GQL, { id: "abc" })
  query Track($id: String!) {
    track(id: $id) { id title artist }
  }
GQL
```

---

## License

MIT License. See [LICENSE](./LICENSE) for details.
