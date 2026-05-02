"""Persistent ("saved") playlists and their folders."""

from __future__ import annotations

from dataclasses import asdict, dataclass

from ..transport import HttpTransport
from ..types import SavedPlaylist, SavedPlaylistFolder

_PLAYLIST_FIELDS = "id name description image folderId trackCount createdAt updatedAt"
_FOLDER_FIELDS = "id name createdAt updatedAt"


@dataclass
class CreatePlaylistInput:
    name: str
    description: str | None = None
    image: str | None = None
    folder_id: str | None = None
    track_ids: list[str] | None = None


@dataclass
class UpdatePlaylistInput:
    name: str
    description: str | None = None
    image: str | None = None
    folder_id: str | None = None


def _camelize(d: dict[str, object]) -> dict[str, object]:
    """folder_id → folderId, track_ids → trackIds, etc."""
    out: dict[str, object] = {}
    for k, v in d.items():
        if v is None:
            continue
        parts = k.split("_")
        camel = parts[0] + "".join(p.title() for p in parts[1:])
        out[camel] = v
    return out


class SavedPlaylistsApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def list(self, folder_id: str | None = None) -> list[SavedPlaylist]:
        data = await self._http.execute(
            "query SavedPlaylists($folderId: String) "
            f"{{ savedPlaylists(folderId: $folderId) {{ {_PLAYLIST_FIELDS} }} }}",
            {"folderId": folder_id},
        )
        return [SavedPlaylist.model_validate(p) for p in data.get("savedPlaylists", [])]

    async def get(self, id: str) -> SavedPlaylist | None:
        data = await self._http.execute(
            "query SavedPlaylist($id: String!) "
            f"{{ savedPlaylist(id: $id) {{ {_PLAYLIST_FIELDS} }} }}",
            {"id": id},
        )
        raw = data.get("savedPlaylist")
        return SavedPlaylist.model_validate(raw) if raw is not None else None

    async def track_ids(self, playlist_id: str) -> list[str]:
        data = await self._http.execute(
            "query SavedPlaylistTrackIds($playlistId: String!) "
            "{ savedPlaylistTrackIds(playlistId: $playlistId) }",
            {"playlistId": playlist_id},
        )
        return list(data.get("savedPlaylistTrackIds", []))

    async def create(self, input: CreatePlaylistInput) -> SavedPlaylist:
        data = await self._http.execute(
            "mutation CreateSavedPlaylist("
            "$name: String!, $description: String, $image: String, "
            "$folderId: String, $trackIds: [String!]) "
            "{ createSavedPlaylist("
            "name: $name, description: $description, image: $image, "
            "folderId: $folderId, trackIds: $trackIds) "
            f"{{ {_PLAYLIST_FIELDS} }} }}",
            _camelize(asdict(input)),
        )
        return SavedPlaylist.model_validate(data["createSavedPlaylist"])

    async def update(self, id: str, input: UpdatePlaylistInput) -> None:
        await self._http.execute(
            "mutation UpdateSavedPlaylist("
            "$id: String!, $name: String!, $description: String, "
            "$image: String, $folderId: String) "
            "{ updateSavedPlaylist("
            "id: $id, name: $name, description: $description, "
            "image: $image, folderId: $folderId) }",
            {"id": id, **_camelize(asdict(input))},
        )

    async def delete(self, id: str) -> None:
        await self._http.execute(
            "mutation DeleteSavedPlaylist($id: String!) { deleteSavedPlaylist(id: $id) }",
            {"id": id},
        )

    async def add_tracks(self, playlist_id: str, track_ids: list[str]) -> None:
        await self._http.execute(
            "mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) "
            "{ addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds) }",
            {"playlistId": playlist_id, "trackIds": track_ids},
        )

    async def remove_track(self, playlist_id: str, track_id: str) -> None:
        await self._http.execute(
            "mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) "
            "{ removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId) }",
            {"playlistId": playlist_id, "trackId": track_id},
        )

    async def play(self, playlist_id: str) -> None:
        await self._http.execute(
            "mutation PlaySavedPlaylist($playlistId: String!) "
            "{ playSavedPlaylist(playlistId: $playlistId) }",
            {"playlistId": playlist_id},
        )

    # --- folders --------------------------------------------------------

    async def folders(self) -> list[SavedPlaylistFolder]:
        data = await self._http.execute(
            f"query PlaylistFolders {{ playlistFolders {{ {_FOLDER_FIELDS} }} }}"
        )
        return [SavedPlaylistFolder.model_validate(f) for f in data.get("playlistFolders", [])]

    async def create_folder(self, name: str) -> SavedPlaylistFolder:
        data = await self._http.execute(
            "mutation CreatePlaylistFolder($name: String!) "
            f"{{ createPlaylistFolder(name: $name) {{ {_FOLDER_FIELDS} }} }}",
            {"name": name},
        )
        return SavedPlaylistFolder.model_validate(data["createPlaylistFolder"])

    async def delete_folder(self, id: str) -> None:
        await self._http.execute(
            "mutation DeletePlaylistFolder($id: String!) { deletePlaylistFolder(id: $id) }",
            {"id": id},
        )
