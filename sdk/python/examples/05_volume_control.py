"""05 — Volume control.

Read current volume (with min/max range) and bump it up by one step.

    uv run python examples/05_volume_control.py          # show + step up
    uv run python examples/05_volume_control.py -3       # step down 3
"""

from __future__ import annotations

import asyncio
import sys

from _client import create_client  # type: ignore[import-not-found]


async def main(delta: int) -> None:
    async with create_client() as client:
        before = await client.sound.get_volume()
        rng = before.max - before.min
        filled = round(((before.volume - before.min) / rng) * 20) if rng > 0 else 0
        bar = "█" * filled + "░" * max(0, 20 - filled)

        print(f"Volume: {before.volume} dB  (range {before.min} … {before.max})")
        print(f"        {bar}")

        after = await client.sound.adjust_volume(delta)
        sign = f"+{delta}" if delta >= 0 else str(delta)
        print(f"\nAdjusted by {sign} → {after} dB")


if __name__ == "__main__":
    delta = int(sys.argv[1]) if len(sys.argv) > 1 else 1
    asyncio.run(main(delta))
