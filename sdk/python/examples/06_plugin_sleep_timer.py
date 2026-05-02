"""06 — Plugin: sleep timer.

Stops playback after N minutes. If the user stops playback manually before the
timer fires, the plugin cancels itself.

    uv run python examples/06_plugin_sleep_timer.py        # default 30 min
    uv run python examples/06_plugin_sleep_timer.py 5      # 5 minutes
"""

from __future__ import annotations

import asyncio
import contextlib
import sys
from datetime import datetime, timedelta

from _client import create_client  # type: ignore[import-not-found]

from rockbox_sdk import PlaybackStatus, PluginContext


class SleepTimer:
    name = "sleep-timer"
    version = "1.0.0"

    def __init__(self, minutes: int) -> None:
        self.minutes = minutes
        self.description = f"Stop playback after {minutes} minute(s)"
        self._task: asyncio.Task[None] | None = None

    def install(self, ctx: PluginContext) -> None:
        fire_at = datetime.now() + timedelta(minutes=self.minutes)
        print(f"💤 Sleep timer armed — will stop playback at {fire_at:%H:%M:%S}")

        async def fire() -> None:
            try:
                await asyncio.sleep(self.minutes * 60)
            except asyncio.CancelledError:
                return
            print("💤 Time's up — stopping playback.")
            await ctx.query("mutation { hardStop }")

        self._task = asyncio.create_task(fire())

        @ctx.events.on("status:changed")
        def cancel_on_stop(status: int) -> None:
            if status == PlaybackStatus.STOPPED and self._task and not self._task.done():
                self._task.cancel()
                print("💤 Playback stopped manually — sleep timer cancelled.")

    def uninstall(self) -> None:
        if self._task and not self._task.done():
            self._task.cancel()


async def main(minutes: int) -> None:
    async with create_client() as client:
        await client.connect()
        await client.use(SleepTimer(minutes))

        print("Plugin installed. Press Ctrl+C to cancel and exit.")
        with contextlib.suppress(asyncio.CancelledError):
            await asyncio.Event().wait()


if __name__ == "__main__":
    minutes = int(sys.argv[1]) if len(sys.argv) > 1 else 30
    try:
        asyncio.run(main(minutes))
    except KeyboardInterrupt:
        print("\nbye")
