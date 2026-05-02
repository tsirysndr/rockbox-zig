"""Playback control: status, transport, and one-shot play helpers."""

from __future__ import annotations

from typing import Any

from ..transport import HttpTransport
from ..types import PlaybackStatus, Track
from ._fragments import TRACK_FIELDS


class PlaybackApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    # --- status ---------------------------------------------------------

    async def raw_status(self) -> int:
        """Raw numeric playback status from the firmware."""
        data = await self._http.execute("query PlaybackStatus { status }")
        return int(data["status"])

    async def status(self) -> PlaybackStatus:
        """Typed playback status."""
        return PlaybackStatus(await self.raw_status())

    async def current_track(self) -> Track | None:
        data = await self._http.execute(
            f"{TRACK_FIELDS} query CurrentTrack {{ currentTrack {{ ...TrackFields }} }}"
        )
        raw = data.get("currentTrack")
        return Track.model_validate(raw) if raw is not None else None

    async def next_track(self) -> Track | None:
        data = await self._http.execute(
            f"{TRACK_FIELDS} query NextTrack {{ nextTrack {{ ...TrackFields }} }}"
        )
        raw = data.get("nextTrack")
        return Track.model_validate(raw) if raw is not None else None

    async def file_position(self) -> int:
        data = await self._http.execute("query FilePosition { getFilePosition }")
        return int(data["getFilePosition"])

    # --- transport ------------------------------------------------------

    async def play(self, elapsed: int = 0, offset: int = 0) -> None:
        await self._http.execute(
            "mutation Play($elapsed: Long!, $offset: Long!) "
            "{ play(elapsed: $elapsed, offset: $offset) }",
            {"elapsed": elapsed, "offset": offset},
        )

    async def pause(self) -> None:
        await self._http.execute("mutation Pause { pause }")

    async def resume(self) -> None:
        await self._http.execute("mutation Resume { resume }")

    async def next(self) -> None:
        await self._http.execute("mutation Next { next }")

    async def previous(self) -> None:
        await self._http.execute("mutation Previous { previous }")

    async def seek(self, position_ms: int) -> None:
        """Seek to an absolute position in milliseconds."""
        await self._http.execute(
            "mutation Seek($newTime: Int!) { fastForwardRewind(newTime: $newTime) }",
            {"newTime": position_ms},
        )

    async def stop(self) -> None:
        await self._http.execute("mutation Stop { hardStop }")

    async def flush_and_reload(self) -> None:
        """Reload and flush the current track queue."""
        await self._http.execute("mutation FlushReload { flushAndReloadTracks }")

    # --- one-shot play helpers -----------------------------------------

    async def play_track(self, path: str) -> None:
        await self._http.execute(
            "mutation PlayTrack($path: String!) { playTrack(path: $path) }",
            {"path": path},
        )

    async def play_album(
        self,
        album_id: str,
        *,
        shuffle: bool | None = None,
        position: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) "
            "{ playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position) }",
            {"albumId": album_id, "shuffle": shuffle, "position": position},
        )

    async def play_artist(
        self,
        artist_id: str,
        *,
        shuffle: bool | None = None,
        position: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlayArtist($artistId: String!, $shuffle: Boolean, $position: Int) "
            "{ playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position) }",
            {"artistId": artist_id, "shuffle": shuffle, "position": position},
        )

    async def play_playlist(
        self,
        playlist_id: str,
        *,
        shuffle: bool | None = None,
        position: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlayPlaylist($playlistId: String!, $shuffle: Boolean, $position: Int) "
            "{ playPlaylist(playlistId: $playlistId, shuffle: $shuffle, position: $position) }",
            {"playlistId": playlist_id, "shuffle": shuffle, "position": position},
        )

    async def play_directory(
        self,
        path: str,
        *,
        recurse: bool | None = None,
        shuffle: bool | None = None,
        position: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlayDirectory("
            "$path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int"
            ") { playDirectory(path: $path, recurse: $recurse, "
            "shuffle: $shuffle, position: $position) }",
            {"path": path, "recurse": recurse, "shuffle": shuffle, "position": position},
        )

    async def play_liked_tracks(
        self,
        *,
        shuffle: bool | None = None,
        position: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlayLikedTracks($shuffle: Boolean, $position: Int) "
            "{ playLikedTracks(shuffle: $shuffle, position: $position) }",
            {"shuffle": shuffle, "position": position},
        )

    async def play_all_tracks(
        self,
        *,
        shuffle: bool | None = None,
        position: int | None = None,
    ) -> None:
        await self._http.execute(
            "mutation PlayAllTracks($shuffle: Boolean, $position: Int) "
            "{ playAllTracks(shuffle: $shuffle, position: $position) }",
            {"shuffle": shuffle, "position": position},
        )

    # ``Any`` to keep mypy quiet about the generic dict shape returned by GraphQL
    _: Any = None
