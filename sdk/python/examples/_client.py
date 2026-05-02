"""Shared client factory used by every example.

Override the host/port via env: ``ROCKBOX_HOST``, ``ROCKBOX_PORT``.
"""

from __future__ import annotations

import os

from rockbox_sdk import RockboxClient


def create_client() -> RockboxClient:
    return RockboxClient(
        host=os.environ.get("ROCKBOX_HOST", "localhost"),
        port=int(os.environ.get("ROCKBOX_PORT", "6062")),
    )


def fmt_time(ms: int) -> str:
    """Format milliseconds as ``M:SS``."""
    total = max(0, ms // 1000)
    return f"{total // 60}:{total % 60:02d}"
