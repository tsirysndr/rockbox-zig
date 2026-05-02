"""Volume and audio level controls."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import VolumeInfo


class SoundApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def get_volume(self) -> VolumeInfo:
        """Current volume with min/max range (firmware steps, typically dB)."""
        data = await self._http.execute("query Volume { volume { volume min max } }")
        return VolumeInfo.model_validate(data["volume"])

    async def adjust_volume(self, steps: int) -> int:
        """Adjust by a relative step count (positive = louder). Returns the new volume."""
        data = await self._http.execute(
            "mutation AdjustVolume($steps: Int!) { adjustVolume(steps: $steps) }",
            {"steps": steps},
        )
        return int(data["adjustVolume"])

    async def volume_up(self) -> int:
        return await self.adjust_volume(1)

    async def volume_down(self) -> int:
        return await self.adjust_volume(-1)
