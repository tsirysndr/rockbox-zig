# Examples

Each example is a runnable script. Start `rockboxd` first, then:

```sh
mix deps.get
mix run examples/01_basic_playback.exs
```

Override the host/port via env: `ROCKBOX_HOST`, `ROCKBOX_PORT`.

| File                              | What it shows                                  |
|-----------------------------------|------------------------------------------------|
| `01_basic_playback.exs`           | Toggle play/pause based on current status      |
| `02_now_playing.exs`              | Real-time event subscriptions                  |
| `03_library_search.exs`           | Full-text search across artists/albums/tracks  |
| `04_queue_management.exs`         | Inspect and modify the live queue              |
| `05_saved_playlists.exs`          | Persistent named playlists                     |
| `06_smart_playlist.exs`           | Builder DSL for rule-based playlists           |
| `07_volume_control.exs`           | Volume up/down                                 |
| `08_eq_config.exs`                | Equalizer configuration                        |
| `09_browse_filesystem.exs`        | Walk `music_dir`                                |
| `10_devices.exs`                  | List Chromecast / AirPlay devices              |
| `11_bluetooth.exs`                | Bluetooth scan / connect (Linux)               |
| `12_plugin_sleep_timer.exs`       | Build a plugin with the `Rockbox.Plugin` behaviour |
| `13_raw_query.exs`                | Escape hatch for one-off GraphQL queries       |

For long-running examples (subscriptions, plugins) use `mix run --no-halt …`.
