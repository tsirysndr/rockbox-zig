"""Tests for the plugin registry."""

from __future__ import annotations

import pytest

from rockbox_sdk import EventEmitter, PluginContext, PluginRegistry


class _Plugin:
    name = "demo"
    version = "1.0.0"
    description = "test"

    def __init__(self) -> None:
        self.installed = False
        self.uninstalled = False

    def install(self, ctx: PluginContext) -> None:
        self.installed = True

    def uninstall(self) -> None:
        self.uninstalled = True


@pytest.mark.asyncio
async def test_register_and_unregister() -> None:
    reg = PluginRegistry()
    plugin = _Plugin()

    async def fake_query(gql: str, variables=None):  # noqa: ANN001, ARG001
        return {}

    ctx = PluginContext(query=fake_query, events=EventEmitter())
    await reg.register(plugin, ctx)
    assert reg.has("demo")
    assert plugin.installed

    await reg.unregister("demo")
    assert not reg.has("demo")
    assert plugin.uninstalled


@pytest.mark.asyncio
async def test_duplicate_register_raises() -> None:
    reg = PluginRegistry()

    async def fake_query(gql: str, variables=None):  # noqa: ANN001, ARG001
        return {}

    ctx = PluginContext(query=fake_query, events=EventEmitter())
    await reg.register(_Plugin(), ctx)
    with pytest.raises(ValueError):
        await reg.register(_Plugin(), ctx)
