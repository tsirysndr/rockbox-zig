# Examples

Each example is a runnable Gleam module. Start `rockboxd` first, then from
this directory:

```sh
gleam deps download
gleam run -m example_01_basic_playback
```

The shared client points at `localhost:6062` — edit `src/helper.gleam` if
your daemon runs elsewhere. Examples that take parameters (search term,
browse path, sleep-timer minutes, …) declare a constant near the top of
the file; tweak it and re-run.

| File                              | What it shows                                          |
|-----------------------------------|--------------------------------------------------------|
| `example_01_basic_playback.gleam` | Toggle play/pause based on current status              |
| `example_02_now_playing.gleam`    | Polling-based current-track watcher                    |
| `example_03_library_search.gleam` | Full-text search across artists/albums/tracks          |
| `example_04_queue_management.gleam` | Inspect and modify the live queue                    |
| `example_05_saved_playlists.gleam` | Persistent named playlists                            |
| `example_06_smart_playlist.gleam` | Rule-based smart playlist via the `rules` builder DSL  |
| `example_07_volume_control.gleam` | Volume up/down                                         |
| `example_08_eq_config.gleam`      | Equalizer configuration via the settings patch builder |
| `example_09_browse_filesystem.gleam` | Walk `music_dir`                                    |
| `example_10_devices.gleam`        | List Chromecast / AirPlay devices                      |
| `example_11_bluetooth.gleam`      | Bluetooth scan / connect (Linux)                       |
| `example_12_sleep_timer.gleam`    | Polling-based sleep timer                              |
| `example_13_raw_query.gleam`      | Escape hatch for one-off GraphQL queries               |

## Differences from the Elixir SDK

The Gleam SDK is request/response-only and doesn't ship the higher-level
abstractions the Elixir SDK has, so a few examples are adapted:

- **02 / now playing** — the Elixir version subscribes to a WebSocket. The
  Gleam version polls `currentTrack` once per second.
- **12 / sleep timer** — the Elixir version uses a `Rockbox.Plugin`
  GenServer. The Gleam version is a simple `process.sleep` loop that bails
  out early if playback was stopped manually.
