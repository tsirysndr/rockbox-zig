# Examples

Each file is a runnable Clojure script. They share `example_client.clj`, which
builds a `RockboxClient` honouring `ROCKBOX_HOST` / `ROCKBOX_PORT` env vars.

```sh
# From sdk/clojure
clj -M:examples -m ex01-basic-playback
clj -M:examples -m ex03-library-search "pink floyd"
ROCKBOX_HOST=192.168.1.42 clj -M:examples -m ex02-now-playing
```

| File                              | What it shows                                         |
|-----------------------------------|-------------------------------------------------------|
| `ex01_basic_playback.clj`         | Pause / seek / resume in one threading-macro chain    |
| `ex02_now_playing.clj`            | Pretty-print the currently playing track              |
| `ex03_library_search.clj`         | Search → play first matching album shuffled           |
| `ex04_queue_management.clj`       | Inspect and modify the live queue                     |
| `ex05_realtime_events.clj`        | WebSocket events with the callback API                |
| `ex06_core_async_events.clj`      | Same events, consumed via `core.async` channels       |
| `ex07_volume_eq.clj`              | Adjust volume + write a 5-band EQ preset              |
| `ex08_browse_filesystem.clj`      | Walk `music_dir` (directories vs files)               |
| `ex09_plugin_scrobbler.clj`       | Toy "scrobbler" plugin via `use-plugin` / event hook  |
| `ex10_smart_playlist.clj`         | Create a smart playlist from a Clojure data rule-set  |

`rockboxd` must be running locally (or specify `ROCKBOX_HOST`) before the
examples can connect.
