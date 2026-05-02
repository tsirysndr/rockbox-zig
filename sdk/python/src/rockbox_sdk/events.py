"""Lightweight async event emitter used by ``RockboxClient``.

Listeners may be sync (``def``) or async (``async def``); the emitter awaits
the latter and calls the former synchronously. There is no Node-style
``EventEmitter`` dependency.
"""

from __future__ import annotations

import asyncio
import inspect
from collections import defaultdict
from collections.abc import Awaitable, Callable
from typing import Any, TypeAlias

# Public event names emitted by the client.
EventName: TypeAlias = str

#: Event names emitted by the SDK. Importing as constants keeps editors honest.
TRACK_CHANGED: EventName = "track:changed"
"""Fires whenever the currently playing track changes. Payload: ``Track``."""

STATUS_CHANGED: EventName = "status:changed"
"""Fires when playback status changes. Payload: ``int`` (raw firmware status)."""

PLAYLIST_CHANGED: EventName = "playlist:changed"
"""Fires when the active playlist changes. Payload: ``Playlist``."""

WS_OPEN: EventName = "ws:open"
"""WebSocket connection opened. No payload."""

WS_CLOSE: EventName = "ws:close"
"""WebSocket connection closed. No payload."""

WS_ERROR: EventName = "ws:error"
"""WebSocket or subscription error. Payload: ``Exception``."""


Listener: TypeAlias = Callable[..., Any] | Callable[..., Awaitable[Any]]


class EventEmitter:
    """Async-aware event emitter.

    Usage:
        emitter = EventEmitter()
        emitter.on("track:changed", lambda t: print(t.title))

        @emitter.on("track:changed")
        async def on_track(track):
            ...
    """

    def __init__(self) -> None:
        self._listeners: dict[str, list[Listener]] = defaultdict(list)

    def on(self, event: str, listener: Listener | None = None) -> Any:
        """Register a listener. Usable as a decorator when ``listener`` is omitted."""
        if listener is None:

            def decorator(fn: Listener) -> Listener:
                self._listeners[event].append(fn)
                return fn

            return decorator
        self._listeners[event].append(listener)
        return self

    def once(self, event: str, listener: Listener | None = None) -> Any:
        """Register a listener that fires at most once."""

        def wrap(fn: Listener) -> Listener:
            async def wrapper(*args: Any, **kwargs: Any) -> None:
                self.off(event, wrapper)
                result = fn(*args, **kwargs)
                if inspect.isawaitable(result):
                    await result

            self._listeners[event].append(wrapper)
            return fn

        if listener is None:
            return wrap
        wrap(listener)
        return self

    def off(self, event: str, listener: Listener) -> EventEmitter:
        """Remove a listener. No-op if it isn't registered."""
        try:
            self._listeners[event].remove(listener)
        except ValueError:
            pass
        return self

    def remove_all_listeners(self, event: str | None = None) -> EventEmitter:
        """Drop every listener, or every listener for ``event`` if specified."""
        if event is None:
            self._listeners.clear()
        else:
            self._listeners.pop(event, None)
        return self

    async def emit(self, event: str, *args: Any) -> None:
        """Notify every listener for ``event``.

        Async listeners are awaited concurrently. Exceptions in any listener are
        re-raised after every other listener has been scheduled — they do not
        prevent other listeners from running.
        """
        listeners = list(self._listeners.get(event, ()))
        if not listeners:
            return

        coros: list[Awaitable[Any]] = []
        for fn in listeners:
            try:
                result = fn(*args)
            except Exception:
                continue
            if inspect.isawaitable(result):
                coros.append(result)

        if coros:
            await asyncio.gather(*coros, return_exceptions=True)
