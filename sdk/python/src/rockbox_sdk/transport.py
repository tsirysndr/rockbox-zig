"""HTTP and WebSocket transport for GraphQL.

``HttpTransport`` is a thin wrapper around ``httpx.AsyncClient`` — one POST per
operation, no caching. ``WsTransport`` implements the
`graphql-transport-ws <https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md>`_
sub-protocol on top of ``websockets``, with infinite reconnects and exponential
backoff.
"""

from __future__ import annotations

import asyncio
import json
import logging
from collections.abc import AsyncIterator, Awaitable, Callable
from typing import Any

import httpx
import websockets
import websockets.exceptions

from .errors import RockboxGraphQLError, RockboxNetworkError

# Type-only — works for websockets >= 11
try:
    from websockets.asyncio.client import (
        ClientConnection as _WsConn,  # type: ignore[import-not-found]
    )
except ImportError:  # pragma: no cover - very old websockets
    from websockets.client import _WsConn as _WsConn  # type: ignore[no-redef]

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# HTTP transport
# ---------------------------------------------------------------------------


class HttpTransport:
    """Async GraphQL-over-HTTP transport.

    The client is created lazily on first use and reused for the lifetime of
    the transport. Call :meth:`aclose` (or use the parent client's context
    manager) to release the underlying connection pool.
    """

    def __init__(self, url: str, *, timeout: float = 30.0) -> None:
        self.url = url
        self._timeout = timeout
        self._client: httpx.AsyncClient | None = None

    def _ensure_client(self) -> httpx.AsyncClient:
        if self._client is None:
            self._client = httpx.AsyncClient(timeout=self._timeout)
        return self._client

    async def execute(
        self,
        query: str,
        variables: dict[str, Any] | None = None,
    ) -> Any:
        """Execute a single GraphQL operation and return the ``data`` payload."""
        client = self._ensure_client()
        try:
            response = await client.post(
                self.url,
                json={"query": query, "variables": variables or {}},
                headers={"Content-Type": "application/json", "Accept": "application/json"},
            )
        except httpx.HTTPError as err:
            raise RockboxNetworkError(f"Failed to reach Rockbox at {self.url}", err) from err

        if response.status_code >= 400:
            raise RockboxNetworkError(
                f"HTTP {response.status_code} {response.reason_phrase}"
            )

        try:
            payload = response.json()
        except json.JSONDecodeError as err:
            raise RockboxNetworkError("Server returned non-JSON response", err) from err

        errors = payload.get("errors") or []
        if errors:
            raise RockboxGraphQLError(errors)
        return payload.get("data") or {}

    async def aclose(self) -> None:
        if self._client is not None:
            await self._client.aclose()
            self._client = None


# ---------------------------------------------------------------------------
# WebSocket transport — graphql-transport-ws subprotocol
# ---------------------------------------------------------------------------


SubscriptionSink = Callable[[dict[str, Any]], Awaitable[None] | None]
ErrorSink = Callable[[Exception], Awaitable[None] | None]


def _is_closed(ws: Any) -> bool:
    """Compatibility shim — both legacy (.closed) and asyncio (.state) APIs."""
    if hasattr(ws, "state"):
        # websockets >= 13: state is an enum, OPEN/CONNECTING are not closed
        from websockets.protocol import State

        return ws.state not in (State.OPEN, State.CONNECTING)
    return bool(getattr(ws, "closed", False))


class WsTransport:
    """Lazy, auto-reconnecting GraphQL subscription client.

    The connection is established on the first ``subscribe`` call and shared
    across all active subscriptions. On disconnect the transport reconnects
    with exponential backoff and re-issues every still-active subscription.
    """

    PROTOCOL = "graphql-transport-ws"

    def __init__(
        self,
        url: str,
        *,
        on_open: Callable[[], Awaitable[None] | None] | None = None,
        on_close: Callable[[], Awaitable[None] | None] | None = None,
        on_error: ErrorSink | None = None,
    ) -> None:
        self.url = url
        self._on_open = on_open
        self._on_close = on_close
        self._on_error = on_error

        self._ws: Any | None = None
        self._reader_task: asyncio.Task[None] | None = None
        self._connect_lock = asyncio.Lock()
        self._next_id = 0
        self._subs: dict[str, _Subscription] = {}
        self._closed = False

    async def subscribe(
        self,
        query: str,
        variables: dict[str, Any] | None,
        on_next: SubscriptionSink,
        on_error: ErrorSink | None = None,
    ) -> Callable[[], Awaitable[None]]:
        """Start a subscription and return a coroutine that cancels it."""
        await self._ensure_connected()

        sub_id = str(self._next_id)
        self._next_id += 1

        sub = _Subscription(
            id=sub_id,
            query=query,
            variables=variables or {},
            on_next=on_next,
            on_error=on_error or self._on_error,
        )
        self._subs[sub_id] = sub
        await self._send_subscribe(sub)

        async def unsubscribe() -> None:
            self._subs.pop(sub_id, None)
            if self._ws is not None and not _is_closed(self._ws):
                try:
                    await self._ws.send(json.dumps({"id": sub_id, "type": "complete"}))
                except Exception:  # noqa: BLE001
                    pass

        return unsubscribe

    async def aclose(self) -> None:
        self._closed = True
        self._subs.clear()
        if self._reader_task is not None:
            self._reader_task.cancel()
            try:
                await self._reader_task
            except (asyncio.CancelledError, Exception):  # noqa: BLE001
                pass
            self._reader_task = None
        if self._ws is not None:
            try:
                await self._ws.close()
            except Exception:  # noqa: BLE001
                pass
            self._ws = None

    # ---- internals ------------------------------------------------------

    async def _ensure_connected(self) -> None:
        if self._closed:
            raise RuntimeError("WsTransport has been closed")
        async with self._connect_lock:
            if self._ws is not None and not _is_closed(self._ws):
                return
            await self._connect_once()

    async def _connect_once(self) -> None:
        try:
            ws = await websockets.connect(self.url, subprotocols=[self.PROTOCOL])
        except Exception as err:  # noqa: BLE001
            raise RockboxNetworkError(f"Failed to open WebSocket to {self.url}", err) from err

        await ws.send(json.dumps({"type": "connection_init"}))
        ack = json.loads(await ws.recv())
        if ack.get("type") != "connection_ack":
            await ws.close()
            raise RockboxNetworkError(f"WebSocket handshake failed: {ack!r}")

        self._ws = ws
        await self._fire(self._on_open)
        self._reader_task = asyncio.create_task(self._reader_loop(ws))

    async def _reader_loop(self, ws: Any) -> None:
        try:
            async for raw in ws:
                try:
                    msg = json.loads(raw)
                except json.JSONDecodeError:
                    continue
                await self._dispatch(msg)
        except (websockets.exceptions.ConnectionClosed, ConnectionError) as err:
            if self._closed:
                return
            await self._fire(self._on_close)
            await self._reconnect_with_backoff(err)
        except Exception as err:  # noqa: BLE001
            await self._fire_error(err)

    async def _dispatch(self, msg: dict[str, Any]) -> None:
        msg_type = msg.get("type")
        sub_id = msg.get("id")
        sub = self._subs.get(sub_id) if sub_id is not None else None

        if msg_type == "next" and sub is not None:
            payload = msg.get("payload") or {}
            try:
                result = sub.on_next(payload)
                if asyncio.iscoroutine(result):
                    await result
            except Exception as err:  # noqa: BLE001
                if sub.on_error is not None:
                    await self._fire_error(err, sub.on_error)
        elif msg_type == "error" and sub is not None:
            err = RockboxGraphQLError(msg.get("payload") or [])
            if sub.on_error is not None:
                await self._fire_error(err, sub.on_error)
        elif msg_type == "complete" and sub_id is not None:
            self._subs.pop(sub_id, None)

    async def _send_subscribe(self, sub: _Subscription) -> None:
        assert self._ws is not None
        await self._ws.send(
            json.dumps(
                {
                    "id": sub.id,
                    "type": "subscribe",
                    "payload": {"query": sub.query, "variables": sub.variables},
                }
            )
        )

    async def _reconnect_with_backoff(self, _err: Exception) -> None:
        attempt = 0
        while not self._closed:
            delay = min(2**attempt, 30)
            await asyncio.sleep(delay)
            attempt += 1
            try:
                await self._connect_once()
            except Exception as err:  # noqa: BLE001
                await self._fire_error(err)
                continue
            # Resubscribe everything we still care about
            for sub in list(self._subs.values()):
                try:
                    await self._send_subscribe(sub)
                except Exception as err:  # noqa: BLE001
                    await self._fire_error(err)
            return

    @staticmethod
    async def _fire(cb: Callable[[], Awaitable[None] | None] | None) -> None:
        if cb is None:
            return
        try:
            res = cb()
            if asyncio.iscoroutine(res):
                await res
        except Exception:  # noqa: BLE001
            logger.exception("event callback raised")

    async def _fire_error(self, err: Exception, sink: ErrorSink | None = None) -> None:
        target = sink or self._on_error
        if target is None:
            logger.warning("ws transport error: %s", err)
            return
        try:
            res = target(err)
            if asyncio.iscoroutine(res):
                await res
        except Exception:  # noqa: BLE001
            logger.exception("error callback raised")


class _Subscription:
    __slots__ = ("id", "query", "variables", "on_next", "on_error")

    def __init__(
        self,
        id: str,
        query: str,
        variables: dict[str, Any],
        on_next: SubscriptionSink,
        on_error: ErrorSink | None,
    ) -> None:
        self.id = id
        self.query = query
        self.variables = variables
        self.on_next = on_next
        self.on_error = on_error


# Re-exported for type checkers — kept out of the public package surface
__all__ = ["HttpTransport", "WsTransport", "AsyncIterator"]
