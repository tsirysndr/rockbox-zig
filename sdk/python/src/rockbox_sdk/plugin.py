"""Plugin protocol — Jellyfin-style install/uninstall lifecycle."""

from __future__ import annotations

from collections.abc import Awaitable
from dataclasses import dataclass
from typing import Any, Protocol, runtime_checkable

from .events import EventEmitter


@dataclass
class PluginContext:
    """Handed to a plugin's :meth:`install`. Lets it issue raw GraphQL and listen for events."""

    query: QueryFn
    events: EventEmitter


class QueryFn(Protocol):
    async def __call__(
        self, gql: str, variables: dict[str, Any] | None = None, /
    ) -> Any: ...


@runtime_checkable
class RockboxPlugin(Protocol):
    """Anything implementing this Protocol can be loaded with :meth:`RockboxClient.use`."""

    name: str
    version: str
    description: str | None

    def install(
        self, context: PluginContext
    ) -> None | Awaitable[None]: ...

    def uninstall(self) -> None | Awaitable[None]:  # type: ignore[empty-body]
        ...


class PluginRegistry:
    """Tracks installed plugins by ``name``. Names must be unique within a client."""

    def __init__(self) -> None:
        self._plugins: dict[str, RockboxPlugin] = {}

    async def register(self, plugin: RockboxPlugin, context: PluginContext) -> None:
        if plugin.name in self._plugins:
            raise ValueError(f"Plugin {plugin.name!r} is already installed")
        result = plugin.install(context)
        if hasattr(result, "__await__"):
            await result  # type: ignore[misc]
        self._plugins[plugin.name] = plugin

    async def unregister(self, name: str) -> None:
        plugin = self._plugins.pop(name, None)
        if plugin is None:
            return
        uninstall = getattr(plugin, "uninstall", None)
        if uninstall is None:
            return
        result = uninstall()
        if hasattr(result, "__await__"):
            await result

    def has(self, name: str) -> bool:
        return name in self._plugins

    def list(self) -> list[RockboxPlugin]:
        return list(self._plugins.values())
