"""Tests for the async EventEmitter."""

from __future__ import annotations

import asyncio

import pytest

from rockbox_sdk import EventEmitter


@pytest.mark.asyncio
async def test_sync_listener() -> None:
    emitter = EventEmitter()
    received: list[int] = []
    emitter.on("ping", lambda x: received.append(x))
    await emitter.emit("ping", 1)
    await emitter.emit("ping", 2)
    assert received == [1, 2]


@pytest.mark.asyncio
async def test_async_listener() -> None:
    emitter = EventEmitter()
    received: list[str] = []

    @emitter.on("greet")
    async def listener(msg: str) -> None:
        await asyncio.sleep(0)
        received.append(msg)

    await emitter.emit("greet", "hi")
    assert received == ["hi"]


@pytest.mark.asyncio
async def test_off_removes_listener() -> None:
    emitter = EventEmitter()
    received: list[int] = []

    def listener(x: int) -> None:
        received.append(x)

    emitter.on("ping", listener)
    await emitter.emit("ping", 1)
    emitter.off("ping", listener)
    await emitter.emit("ping", 2)
    assert received == [1]


@pytest.mark.asyncio
async def test_remove_all_listeners() -> None:
    emitter = EventEmitter()
    received: list[int] = []
    emitter.on("a", lambda x: received.append(x))
    emitter.on("b", lambda x: received.append(x * 10))
    emitter.remove_all_listeners()
    await emitter.emit("a", 1)
    await emitter.emit("b", 2)
    assert received == []
