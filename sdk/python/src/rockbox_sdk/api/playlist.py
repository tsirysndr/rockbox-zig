"""Active playlist (queue) management."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import InsertPosition, Playlist
from ._fragments import TRACK_FIELDS


class PlaylistApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    async def current(self) -> Playlist:
        data = await self._http.execute(
            f"{TRACK_FIELDS} query CurrentPlaylist {{ playlistGetCurrent {{ "
            "amount index maxPlaylistSize firstIndex "
            "lastInsertPos seed lastShuffledStart "
            "tracks { ...TrackFields } } }}"
        )
        return Playlist.model_validate(data["playlistGetCurrent"])

    async def amount(self) -> int:
        data = await self._http.execute("query PlaylistAmount { playlistAmount }")
        return int(data["playlistAmount"])

    # --- queue mutations -----------------------------------------------

    async def insert_tracks(
        self,
        paths: list[str],
        position: InsertPosition = InsertPosition.NEXT,
        playlist_id: str | None = None,
    ) -> None:
        """Insert track paths or IDs into the queue."""
        await self._http.execute(
            "mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) "
            "{ insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks) }",
            {"playlistId": playlist_id, "position": int(position), "tracks": paths},
        )

    async def insert_directory(
        self,
        directory: str,
        position: InsertPosition = InsertPosition.LAST,
        playlist_id: str | None = None,
    ) -> None:
        await self._http.execute(
            "mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) "
            "{ insertDirectory(playlistId: $playlistId, "
            "position: $position, directory: $directory) }",
            {"playlistId": playlist_id, "position": int(position), "directory": directory},
        )

    async def insert_album(
        self, album_id: str, position: InsertPosition = InsertPosition.LAST
    ) -> None:
        await self._http.execute(
            "mutation InsertAlbum($albumId: String!, $position: Int!) "
            "{ insertAlbum(albumId: $albumId, position: $position) }",
            {"albumId": album_id, "position": int(position)},
        )

    async def remove_track(self, index: int) -> None:
        await self._http.execute(
            "mutation RemoveTrack($index: Int!) { playlistRemoveTrack(index: $index) }",
            {"index": index},
        )

    async def clear(self) -> None:
        await self._http.execute("mutation ClearPlaylist { playlistRemoveAllTracks }")

    async def shuffle(self) -> None:
        await self._http.execute("mutation ShufflePlaylist { shufflePlaylist }")

    async def create(self, name: str, tracks: list[str]) -> None:
        """Create and start a new temporary playlist (replaces the current queue)."""
        await self._http.execute(
            "mutation CreatePlaylist($name: String!, $tracks: [String!]!) "
            "{ playlistCreate(name: $name, tracks: $tracks) }",
            {"name": name, "tracks": tracks},
        )

    async def start(
        self,
        *,
        start_index: int | None = None,
        elapsed: int | None = None,
        offset: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlaylistStart($startIndex: Int, $elapsed: Int, $offset: Int) "
            "{ playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset) }",
            {"startIndex": start_index, "elapsed": elapsed, "offset": offset},
        )

    async def resume(self) -> None:
        await self._http.execute("mutation PlaylistResume { playlistResume }")
