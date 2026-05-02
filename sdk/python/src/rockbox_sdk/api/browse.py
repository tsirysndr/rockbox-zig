"""Filesystem browser (also surfaces UPnP entries)."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import Entry


class BrowseApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def entries(self, path: str | None = None) -> list[Entry]:
        data = await self._http.execute(
            "query Browse($path: String) "
            "{ treeGetEntries(path: $path) "
            "{ name attr timeWrite customaction displayName } }",
            {"path": path},
        )
        return [Entry.model_validate(e) for e in data.get("treeGetEntries", [])]

    async def directories(self, path: str | None = None) -> list[Entry]:
        return [e for e in await self.entries(path) if e.is_directory]

    async def files(self, path: str | None = None) -> list[Entry]:
        return [e for e in await self.entries(path) if not e.is_directory]
