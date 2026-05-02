# rockbox-sdk

[![PyPI](https://img.shields.io/pypi/v/rockbox-sdk?style=flat-square&logo=pypi&logoColor=white&color=3775A9)](https://pypi.org/project/rockbox-sdk/)
[![Python](https://img.shields.io/pypi/pyversions/rockbox-sdk?style=flat-square&logo=python&logoColor=white&color=3776AB)](https://pypi.org/project/rockbox-sdk/)
[![Downloads](https://img.shields.io/pypi/dm/rockbox-sdk?style=flat-square&logo=pypi&logoColor=white&color=blue)](https://pypi.org/project/rockbox-sdk/)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
[![AsyncIO](https://img.shields.io/badge/asyncio-ready-brightgreen?style=flat-square&logo=python&logoColor=white)](https://docs.python.org/3/library/asyncio.html)
[![Typed](https://img.shields.io/badge/typed-pydantic-e92063?style=flat-square&logo=pydantic&logoColor=white)](https://docs.pydantic.dev/)
[![Ruff](https://img.shields.io/badge/lint-ruff-261230?style=flat-square&logo=ruff&logoColor=white)](https://github.com/astral-sh/ruff)
[![mypy](https://img.shields.io/badge/mypy-strict-1F5082?style=flat-square)](http://mypy-lang.org/)
[![GitHub](https://img.shields.io/badge/github-rockbox--zig-181717?style=flat-square&logo=github&logoColor=white)](https://github.com/tsirysndr/rockbox-zig)

Async Python SDK for [Rockbox Zig](https://github.com/tsirysndr/rockbox-zig) — a typed, batteries-included
client for the GraphQL API exposed by `rockboxd`.

```python
import asyncio
from rockbox_sdk import RockboxClient, PlaybackStatus

async def main():
    async with RockboxClient(host="localhost") as client:
        track = await client.playback.current_track()
        if track:
            print(f"Now: {track.title} — {track.artist}")
        if await client.playback.status() == PlaybackStatus.PAUSED:
            await client.playback.resume()

asyncio.run(main())
```

## Highlights

- **Async-first** — built on `httpx` + `websockets`. Use `await` everywhere.
- **Domain-namespaced API** — `client.playback.*`, `client.library.*`, `client.sound.*`, …
- **Typed responses** — every reply is a Pydantic model with snake_case fields.
- **Real-time events** — `connect()` opens a WebSocket and forwards
  `track:changed` / `status:changed` / `playlist:changed` to listeners.
- **Builder API** — `RockboxClient.builder().host(...).port(...).build()`.
- **Plugin system** — Jellyfin-style install/uninstall lifecycle.
- **Python-friendly** — context manager, decorator listeners, dataclass inputs.

## Install

```sh
uv add rockbox-sdk
# or
pip install rockbox-sdk
```

Requires Python 3.10+ and a running `rockboxd` (default port 6062).

## Try it in the REPL

The SDK is async-first. The recommended REPL is **IPython** — `await` works at
the top level, and you get tab-completion on models, inline docs with `?`, and
`%timeit` for benchmarking:

```sh
uv run ipython
```

```python
In [1]: from rockbox_sdk import RockboxClient, PlaybackStatus
In [2]: client = RockboxClient(host="localhost", port=6062)
In [3]: await client.playback.status()
Out[3]: <PlaybackStatus.PLAYING: 1>
In [4]: track = await client.playback.current_track()
In [5]: track.title, track.artist
Out[5]: ('Money', 'Pink Floyd')
In [6]: await client.sound.get_volume()
Out[6]: VolumeInfo(volume=-12, min=-74, max=6)
In [7]: await client.library.search("daft punk")
In [8]: await client.aclose()
```

You can also test offline — models, enums, and the builder don't need a server:

```python
In [1]: from rockbox_sdk import RockboxClient, Track, InsertPosition
In [2]: Track.model_validate({"title": "Money", "albumArt": "x.jpg"}).album_art
Out[2]: 'x.jpg'
In [3]: RockboxClient.builder().host("nas.local").build()._config.resolve_http_url()
Out[3]: 'http://nas.local:6062/graphql'
```

If you prefer the stdlib REPL, `python -m asyncio` also supports top-level
`await`, or wrap each call in `asyncio.run(...)` in a plain `python` session:

```sh
uv run python -m asyncio
```

```python
>>> from rockbox_sdk import RockboxClient
>>> client = RockboxClient()
>>> await client.playback.status()
```

Subscriptions (`await client.connect()`) keep firing in the background between
prompts in both REPLs.

## Configure

```python
from rockbox_sdk import RockboxClient

# Direct kwargs
client = RockboxClient(host="192.168.1.42", port=6062)

# Or fluent builder
client = (
    RockboxClient.builder()
    .host("nas.local")
    .port(6062)
    .timeout(15)
    .build()
)

# Or full URL override
client = RockboxClient(
    http_url="http://nas.local:6062/graphql",
    ws_url="ws://nas.local:6062/graphql",
)
```

Always call `await client.aclose()` when you're done — or use it as an
async context manager:

```python
async with RockboxClient() as client:
    ...
```

## Domains

| Namespace                  | What it does                                           |
| -------------------------- | ------------------------------------------------------ |
| `client.playback`          | Transport (`play`/`pause`/`seek`), play helpers        |
| `client.library`           | Albums, artists, tracks, search, likes, scan           |
| `client.playlist`          | The active queue (insert/remove/shuffle/start)         |
| `client.saved_playlists`   | Persistent playlists & folders                         |
| `client.smart_playlists`   | Rule-based playlists & listening stats                 |
| `client.sound`             | Volume control                                         |
| `client.settings`          | Global EQ / replaygain / crossfade / shuffle / …       |
| `client.system`            | Version, runtime info                                  |
| `client.browse`            | Filesystem & UPnP browser                              |
| `client.devices`           | Cast / source device discovery                         |
| `client.bluetooth`         | Bluetooth pairing & scanning (Linux only)              |

## Real-time events

```python
from rockbox_sdk import RockboxClient, TRACK_CHANGED, STATUS_CHANGED

async with RockboxClient() as client:
    await client.connect()  # opens the WebSocket

    @client.on(TRACK_CHANGED)
    async def on_track(track):
        print(f"▶ {track.title} — {track.artist}")

    @client.on(STATUS_CHANGED)
    def on_status(raw_status):
        print(f"◐ status = {raw_status}")

    await asyncio.Event().wait()  # run forever
```

Convenience wrappers exist (`client.on_track_changed(...)`,
`client.on_status_changed(...)`, `client.on_playlist_changed(...)`).

## Plugins

A plugin is anything matching the `RockboxPlugin` protocol — a name, version,
`install(context)`, and optionally `uninstall()`:

```python
from rockbox_sdk import RockboxClient, PlaybackStatus, RockboxPlugin

class SleepTimer:
    name = "sleep-timer"
    version = "1.0.0"
    description = "Stop playback after N minutes"

    def __init__(self, minutes: int) -> None:
        self.minutes = minutes
        self._task: asyncio.Task | None = None

    def install(self, ctx):
        async def fire():
            await asyncio.sleep(self.minutes * 60)
            await ctx.query("mutation { hardStop }")

        self._task = asyncio.create_task(fire())

        @ctx.events.on("status:changed")
        def cancel_on_stop(status: int):
            if status == PlaybackStatus.STOPPED and self._task:
                self._task.cancel()

    def uninstall(self):
        if self._task:
            self._task.cancel()

async with RockboxClient() as client:
    await client.connect()
    await client.use(SleepTimer(30))
```

## Raw GraphQL escape hatch

```python
data = await client.query(
    "query Volume { volume { volume min max } }"
)
```

## Examples

See `examples/` for runnable scripts mirroring the TypeScript SDK examples:

- `01-basic-playback.py`
- `02-now-playing.py`
- `03-library-search.py`
- `04-queue-management.py`
- `05-volume-control.py`
- `06-plugin-sleep-timer.py`

Run with:

```sh
uv run python examples/01_basic_playback.py
```

---

## License

MIT License. See [LICENSE](./LICENSE) for details.
