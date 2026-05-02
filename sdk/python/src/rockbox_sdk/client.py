"""``RockboxClient`` — async, top-level entry point.

Inspired by the same systems as the TypeScript SDK:
- Mopidy: domain-namespaced API (``client.playback.play()``, ``client.library.search()``)
- Jellyfin: plugin lifecycle (install / uninstall)
- Kodi: rich device & playlist management
- Navidrome: clean typed search & library queries
"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from types import TracebackType
from typing import Any

from .api import (
    BluetoothApi,
    BrowseApi,
    DevicesApi,
    LibraryApi,
    PlaybackApi,
    PlaylistApi,
    SavedPlaylistsApi,
    SettingsApi,
    SmartPlaylistsApi,
    SoundApi,
    SystemApi,
)
from .events import (
    PLAYLIST_CHANGED,
    STATUS_CHANGED,
    TRACK_CHANGED,
    WS_ERROR,
    EventEmitter,
    Listener,
)
from .plugin import PluginContext, PluginRegistry, RockboxPlugin
from .transport import HttpTransport, WsTransport
from .types import Playlist, Track


@dataclass
class RockboxClientConfig:
    """Connection settings. Use :meth:`RockboxClient.builder` for a fluent constructor."""

    host: str = "localhost"
    """Hostname or IP of the rockboxd instance."""

    port: int = 6062
    """GraphQL HTTP/WS port."""

    http_url: str | None = None
    """Override the full HTTP URL. Takes precedence over ``host``/``port``."""

    ws_url: str | None = None
    """Override the full WebSocket URL. Takes precedence over ``host``/``port``."""

    timeout: float = 30.0
    """Per-request HTTP timeout in seconds."""

    def resolve_http_url(self) -> str:
        return self.http_url or f"http://{self.host}:{self.port}/graphql"

    def resolve_ws_url(self) -> str:
        return self.ws_url or f"ws://{self.host}:{self.port}/graphql"


class RockboxClientBuilder:
    """Fluent builder for :class:`RockboxClient`.

    Example::

        client = (
            RockboxClient.builder()
            .host("192.168.1.42")
            .port(6062)
            .timeout(15)
            .build()
        )
    """

    def __init__(self) -> None:
        self._cfg = RockboxClientConfig()

    def host(self, host: str) -> RockboxClientBuilder:
        self._cfg.host = host
        return self

    def port(self, port: int) -> RockboxClientBuilder:
        self._cfg.port = port
        return self

    def http_url(self, url: str) -> RockboxClientBuilder:
        self._cfg.http_url = url
        return self

    def ws_url(self, url: str) -> RockboxClientBuilder:
        self._cfg.ws_url = url
        return self

    def timeout(self, seconds: float) -> RockboxClientBuilder:
        self._cfg.timeout = seconds
        return self

    def build(self) -> RockboxClient:
        return RockboxClient(self._cfg)


class RockboxClient(EventEmitter):
    """Async Rockbox client.

    Two ways to construct:

        # Direct keyword args:
        client = RockboxClient(host="localhost", port=6062)

        # Fluent builder:
        client = RockboxClient.builder().host("nas.local").build()

    Use as an async context manager to clean up the HTTP pool and WebSocket
    automatically::

        async with RockboxClient() as client:
            await client.playback.play()
    """

    def __init__(
        self,
        config: RockboxClientConfig | None = None,
        *,
        host: str | None = None,
        port: int | None = None,
        http_url: str | None = None,
        ws_url: str | None = None,
        timeout: float | None = None,
    ) -> None:
        super().__init__()
        cfg = config or RockboxClientConfig()
        if host is not None:
            cfg.host = host
        if port is not None:
            cfg.port = port
        if http_url is not None:
            cfg.http_url = http_url
        if ws_url is not None:
            cfg.ws_url = ws_url
        if timeout is not None:
            cfg.timeout = timeout
        self._config = cfg

        self._http = HttpTransport(cfg.resolve_http_url(), timeout=cfg.timeout)
        self._ws = WsTransport(
            cfg.resolve_ws_url(),
            on_error=self._forward_ws_error,
        )
        self._plugins = PluginRegistry()
        self._unsubs: list[Callable[[], Awaitable[None]]] = []

        # Domain APIs
        self.playback = PlaybackApi(self._http)
        self.library = LibraryApi(self._http)
        self.playlist = PlaylistApi(self._http)
        self.saved_playlists = SavedPlaylistsApi(self._http)
        self.smart_playlists = SmartPlaylistsApi(self._http)
        self.sound = SoundApi(self._http)
        self.settings = SettingsApi(self._http)
        self.system = SystemApi(self._http)
        self.browse = BrowseApi(self._http)
        self.devices = DevicesApi(self._http)
        self.bluetooth = BluetoothApi(self._http)

    # ---- builder & context manager ------------------------------------

    @classmethod
    def builder(cls) -> RockboxClientBuilder:
        return RockboxClientBuilder()

    async def __aenter__(self) -> RockboxClient:
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        await self.aclose()

    # ---- subscriptions ------------------------------------------------

    async def connect(self) -> RockboxClient:
        """Open the WebSocket and start forwarding subscription events.

        Idempotent: calling twice has no effect.
        """
        if self._unsubs:
            return self

        track_unsub = await self._ws.subscribe(
            """
            subscription CurrentlyPlaying {
              currentlyPlayingSong {
                id title artist album albumArt albumId artistId path length elapsed
              }
            }
            """,
            None,
            self._on_track_payload,
            self._forward_ws_error,
        )

        status_unsub = await self._ws.subscribe(
            "subscription PlaybackStatus { playbackStatus { status } }",
            None,
            self._on_status_payload,
            self._forward_ws_error,
        )

        playlist_unsub = await self._ws.subscribe(
            """
            subscription PlaylistChanged {
              playlistChanged {
                amount index maxPlaylistSize firstIndex lastInsertPos
                seed lastShuffledStart
                tracks { id title artist album path length albumArt }
              }
            }
            """,
            None,
            self._on_playlist_payload,
            self._forward_ws_error,
        )

        self._unsubs = [track_unsub, status_unsub, playlist_unsub]
        return self

    async def disconnect(self) -> None:
        """Tear down all subscriptions and close the WebSocket."""
        for unsub in self._unsubs:
            try:
                await unsub()
            except Exception:  # noqa: BLE001
                pass
        self._unsubs = []
        await self._ws.aclose()

    async def aclose(self) -> None:
        """Close all transports. Use this (or the context manager) at shutdown."""
        await self.disconnect()
        await self._http.aclose()

    # ---- payload handlers --------------------------------------------

    async def _on_track_payload(self, payload: dict[str, Any]) -> None:
        data = (payload.get("data") or {}).get("currentlyPlayingSong")
        if data is not None:
            await self.emit(TRACK_CHANGED, Track.model_validate(data))

    async def _on_status_payload(self, payload: dict[str, Any]) -> None:
        data = (payload.get("data") or {}).get("playbackStatus")
        if data is not None and "status" in data:
            await self.emit(STATUS_CHANGED, int(data["status"]))

    async def _on_playlist_payload(self, payload: dict[str, Any]) -> None:
        data = (payload.get("data") or {}).get("playlistChanged")
        if data is not None:
            await self.emit(PLAYLIST_CHANGED, Playlist.model_validate(data))

    async def _forward_ws_error(self, err: Exception) -> None:
        await self.emit(WS_ERROR, err)

    # ---- ergonomic event helpers (typed convenience) -----------------

    def on_track_changed(self, listener: Listener) -> RockboxClient:
        self.on(TRACK_CHANGED, listener)
        return self

    def on_status_changed(self, listener: Listener) -> RockboxClient:
        self.on(STATUS_CHANGED, listener)
        return self

    def on_playlist_changed(self, listener: Listener) -> RockboxClient:
        self.on(PLAYLIST_CHANGED, listener)
        return self

    # ---- plugins ------------------------------------------------------

    async def use(self, plugin: RockboxPlugin) -> RockboxClient:
        """Register a plugin. Plugins are identified by their ``name``."""
        await self._plugins.register(
            plugin,
            PluginContext(query=self.query, events=self),
        )
        return self

    async def unuse(self, name: str) -> RockboxClient:
        await self._plugins.unregister(name)
        return self

    def installed_plugins(self) -> list[RockboxPlugin]:
        return self._plugins.list()

    # ---- raw escape hatch --------------------------------------------

    async def query(self, gql: str, variables: dict[str, Any] | None = None) -> Any:
        """Run an arbitrary GraphQL operation and return ``data``."""
        return await self._http.execute(gql, variables)

    # ---- blocking helpers --------------------------------------------

    def run(self, coro: Awaitable[Any]) -> Any:
        """Helper for sync scripts: ``client.run(client.playback.play())``.

        This is just ``asyncio.run(...)``; useful in REPLs and short scripts.
        It cannot be called from inside an already-running event loop.
        """
        try:
            asyncio.get_running_loop()
        except RuntimeError:
            return asyncio.run(coro)  # type: ignore[arg-type]
        raise RuntimeError(
            "RockboxClient.run() cannot be called from a running event loop; "
            "await the coroutine directly."
        )
