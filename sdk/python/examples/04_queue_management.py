"""04 — Queue management.

Show the current queue, then insert tracks from the first album in the library
at the end of the queue.

    uv run python examples/04_queue_management.py
"""

from __future__ import annotations

import asyncio

from _client import create_client  # type: ignore[import-not-found]

from rockbox_sdk import InsertPosition


async def main() -> None:
    async with create_client() as client:
        queue = await client.playlist.current()
        print(f"Queue has {queue.amount} tracks (index {queue.index}):")
        for i, t in enumerate(queue.tracks[:5]):
            marker = "▶" if i == queue.index else " "
            print(f"  {marker} {i:>3}  {t.title} — {t.artist}")
        if queue.amount > 5:
            print(f"     … and {queue.amount - 5} more")

        albums = await client.library.albums()
        if albums:
            album = albums[0]
            print(f"\nAppending album: {album.title} — {album.artist}")
            await client.playlist.insert_album(album.id, InsertPosition.LAST)
            print("→ done")


if __name__ == "__main__":
    asyncio.run(main())
