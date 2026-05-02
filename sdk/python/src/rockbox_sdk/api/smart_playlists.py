"""Smart (rule-based) playlists and listening stats."""

from __future__ import annotations

from dataclasses import asdict, dataclass

from ..transport import HttpTransport
from ..types import SmartPlaylist, TrackStats
from .saved_playlists import _camelize

_SMART_FIELDS = (
    "id name description image folderId isSystem rules createdAt updatedAt"
)


@dataclass
class CreateSmartPlaylistInput:
    name: str
    rules: str
    description: str | None = None
    image: str | None = None
    folder_id: str | None = None


@dataclass
class UpdateSmartPlaylistInput:
    name: str
    rules: str
    description: str | None = None
    image: str | None = None
    folder_id: str | None = None


class SmartPlaylistsApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def list(self) -> list[SmartPlaylist]:
        data = await self._http.execute(
            f"query SmartPlaylists {{ smartPlaylists {{ {_SMART_FIELDS} }} }}"
        )
        return [SmartPlaylist.model_validate(p) for p in data.get("smartPlaylists", [])]

    async def get(self, id: str) -> SmartPlaylist | None:
        data = await self._http.execute(
            "query SmartPlaylist($id: String!) "
            f"{{ smartPlaylist(id: $id) {{ {_SMART_FIELDS} }} }}",
            {"id": id},
        )
        raw = data.get("smartPlaylist")
        return SmartPlaylist.model_validate(raw) if raw is not None else None

    async def track_ids(self, id: str) -> list[str]:
        data = await self._http.execute(
            "query SmartPlaylistTrackIds($id: String!) { smartPlaylistTrackIds(id: $id) }",
            {"id": id},
        )
        return list(data.get("smartPlaylistTrackIds", []))

    async def create(self, input: CreateSmartPlaylistInput) -> SmartPlaylist:
        data = await self._http.execute(
            "mutation CreateSmartPlaylist("
            "$name: String!, $rules: String!, $description: String, "
            "$image: String, $folderId: String) "
            "{ createSmartPlaylist("
            "name: $name, rules: $rules, description: $description, "
            "image: $image, folderId: $folderId) "
            f"{{ {_SMART_FIELDS} }} }}",
            _camelize(asdict(input)),
        )
        return SmartPlaylist.model_validate(data["createSmartPlaylist"])

    async def update(self, id: str, input: UpdateSmartPlaylistInput) -> None:
        await self._http.execute(
            "mutation UpdateSmartPlaylist("
            "$id: String!, $name: String!, $rules: String!, "
            "$description: String, $image: String, $folderId: String) "
            "{ updateSmartPlaylist("
            "id: $id, name: $name, rules: $rules, description: $description, "
            "image: $image, folderId: $folderId) }",
            {"id": id, **_camelize(asdict(input))},
        )

    async def delete(self, id: str) -> None:
        await self._http.execute(
            "mutation DeleteSmartPlaylist($id: String!) { deleteSmartPlaylist(id: $id) }",
            {"id": id},
        )

    async def play(self, id: str) -> None:
        await self._http.execute(
            "mutation PlaySmartPlaylist($id: String!) { playSmartPlaylist(id: $id) }",
            {"id": id},
        )

    # --- listening stats -----------------------------------------------

    async def track_stats(self, track_id: str) -> TrackStats | None:
        data = await self._http.execute(
            "query TrackStats($trackId: String!) "
            "{ trackStats(trackId: $trackId) "
            "{ trackId playCount skipCount lastPlayed lastSkipped updatedAt } }",
            {"trackId": track_id},
        )
        raw = data.get("trackStats")
        return TrackStats.model_validate(raw) if raw is not None else None

    async def record_played(self, track_id: str) -> None:
        await self._http.execute(
            "mutation RecordTrackPlayed($trackId: String!) "
            "{ recordTrackPlayed(trackId: $trackId) }",
            {"trackId": track_id},
        )

    async def record_skipped(self, track_id: str) -> None:
        await self._http.execute(
            "mutation RecordTrackSkipped($trackId: String!) "
            "{ recordTrackSkipped(trackId: $trackId) }",
            {"trackId": track_id},
        )
