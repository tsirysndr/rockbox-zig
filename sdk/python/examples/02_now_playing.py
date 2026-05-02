"""02 — Now playing (real-time subscriptions).

Opens a WebSocket and prints every track / status / queue change. Ctrl+C exits.

    uv run python examples/02_now_playing.py
"""

from __future__ import annotations

import asyncio
import contextlib

from _client import create_client  # type: ignore[import-not-found]

from rockbox_sdk import PlaybackStatus, Playlist, Track


async def main() -> None:
    async with create_client() as client:
        await client.connect()

        @client.on_track_changed
        def _track(track: Track) -> None:
            print(f"▶  {track.title} — {track.artist}  [{track.album}]")

        @client.on_status_changed
        def _status(raw: int) -> None:
            try:
                label = PlaybackStatus(raw).name
            except ValueError:
                label = str(raw)
            print(f"◐  {label}")

        @client.on_playlist_changed
        def _queue(q: Playlist) -> None:
            print(f"☰  queue updated — {q.amount} tracks (index {q.index})")

        @client.on("ws:error")
        def _err(err: Exception) -> None:
            print(f"✗  websocket error: {err}")

        print("Listening for events. Press Ctrl+C to exit.")
        with contextlib.suppress(asyncio.CancelledError):
            await asyncio.Event().wait()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nDisconnecting...")
