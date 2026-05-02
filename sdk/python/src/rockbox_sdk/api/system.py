"""System-level info: version, runtime status."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import SystemStatus


class SystemApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def version(self) -> str:
        data = await self._http.execute("query Version { rockboxVersion }")
        return str(data["rockboxVersion"])

    async def status(self) -> SystemStatus:
        data = await self._http.execute(
            "query GlobalStatus { globalStatus { "
            "resumeIndex resumeCrc32 resumeElapsed resumeOffset "
            "runtime topruntime dircacheSize "
            "lastScreen viewerIconCount lastVolumeChange } }"
        )
        return SystemStatus.model_validate(data["globalStatus"])
