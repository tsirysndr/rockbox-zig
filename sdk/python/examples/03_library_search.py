"""03 — Search the library.

Search the library, print a summary, start playing the first matching album.

    uv run python examples/03_library_search.py "pink floyd"
"""

from __future__ import annotations

import asyncio
import sys

from _client import create_client  # type: ignore[import-not-found]


async def main(term: str) -> None:
    async with create_client() as client:
        results = await client.library.search(term)

        print(f'Search: "{term}"')
        print(f"  Artists      : {len(results.artists)}")
        print(f"  Albums       : {len(results.albums)}")
        print(f"  Tracks       : {len(results.tracks)}")
        print(f"  Liked albums : {len(results.liked_albums)}")
        print(f"  Liked tracks : {len(results.liked_tracks)}\n")

        print("Top albums:")
        for a in results.albums[:5]:
            copyright = f" © {a.copyright_message}" if a.copyright_message else ""
            print(f"  • {a.title} — {a.artist} ({a.year}){copyright}")

        if results.albums:
            first = results.albums[0]
            print(f"\nPlaying: {first.title}")
            await client.playback.play_album(first.id, shuffle=False)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("usage: python examples/03_library_search.py <search term>")
        sys.exit(1)
    asyncio.run(main(sys.argv[1]))
