"""01 — Basic playback.

Inspect the current track, then either pause or resume based on state.
Idempotent: run twice and it toggles between Playing and Paused.

    uv run python examples/01_basic_playback.py
"""

from __future__ import annotations

import asyncio

from _client import create_client, fmt_time  # type: ignore[import-not-found]

from rockbox_sdk import PlaybackStatus


async def main() -> None:
    async with create_client() as client:
        status = await client.playback.status()
        print(f"Status: {status.name}")

        track = await client.playback.current_track()
        if track:
            pct = round((track.elapsed / track.length) * 100) if track.length else 0
            print(f"Now: {track.title} — {track.artist}")
            print(f"     {fmt_time(track.elapsed)} / {fmt_time(track.length)} ({pct}%)")
        else:
            print("Nothing is playing.")

        if status == PlaybackStatus.PLAYING:
            await client.playback.pause()
            print("→ paused")
        elif status == PlaybackStatus.PAUSED:
            await client.playback.resume()
            print("→ resumed")


if __name__ == "__main__":
    asyncio.run(main())
