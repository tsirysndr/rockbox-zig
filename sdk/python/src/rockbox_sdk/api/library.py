"""Library: albums, artists, tracks, search, likes, scan."""

from __future__ import annotations

from ..transport import HttpTransport
from ..types import Album, Artist, SearchResults, Track
from ._fragments import ALBUM_FIELDS, ARTIST_FIELDS, TRACK_FIELDS


class LibraryApi:
    def __init__(self, http: HttpTransport) -> None:
        self._http = http

    # --- albums ---------------------------------------------------------

    async def albums(self) -> list[Album]:
        data = await self._http.execute(
            f"{ALBUM_FIELDS} query Albums "
            "{ albums { ...AlbumFields tracks { id title path length albumArt } } }"
        )
        return [Album.model_validate(a) for a in data.get("albums", [])]

    async def album(self, id: str) -> Album | None:
        data = await self._http.execute(
            f"{TRACK_FIELDS} {ALBUM_FIELDS} "
            "query Album($id: String!) "
            "{ album(id: $id) { ...AlbumFields tracks { ...TrackFields } } }",
            {"id": id},
        )
        raw = data.get("album")
        return Album.model_validate(raw) if raw is not None else None

    async def liked_albums(self) -> list[Album]:
        data = await self._http.execute(
            f"{ALBUM_FIELDS} query LikedAlbums {{ likedAlbums {{ ...AlbumFields }} }}"
        )
        return [Album.model_validate(a) for a in data.get("likedAlbums", [])]

    async def like_album(self, id: str) -> None:
        await self._http.execute(
            "mutation LikeAlbum($id: String!) { likeAlbum(id: $id) }", {"id": id}
        )

    async def unlike_album(self, id: str) -> None:
        await self._http.execute(
            "mutation UnlikeAlbum($id: String!) { unlikeAlbum(id: $id) }", {"id": id}
        )

    # --- artists --------------------------------------------------------

    async def artists(self) -> list[Artist]:
        data = await self._http.execute(
            f"{ARTIST_FIELDS} query Artists "
            "{ artists { ...ArtistFields albums { id title albumArt year } } }"
        )
        return [Artist.model_validate(a) for a in data.get("artists", [])]

    async def artist(self, id: str) -> Artist | None:
        data = await self._http.execute(
            f"{ARTIST_FIELDS} {TRACK_FIELDS} "
            "query Artist($id: String!) { artist(id: $id) { "
            "...ArtistFields "
            "albums { id title albumArt year yearString md5 artistId "
            "tracks { id title path length } } "
            "tracks { ...TrackFields } } }",
            {"id": id},
        )
        raw = data.get("artist")
        return Artist.model_validate(raw) if raw is not None else None

    # --- tracks ---------------------------------------------------------

    async def tracks(self) -> list[Track]:
        data = await self._http.execute(
            f"{TRACK_FIELDS} query Tracks {{ tracks {{ ...TrackFields }} }}"
        )
        return [Track.model_validate(t) for t in data.get("tracks", [])]

    async def track(self, id: str) -> Track | None:
        data = await self._http.execute(
            f"{TRACK_FIELDS} query Track($id: String!) "
            "{ track(id: $id) { ...TrackFields } }",
            {"id": id},
        )
        raw = data.get("track")
        return Track.model_validate(raw) if raw is not None else None

    async def liked_tracks(self) -> list[Track]:
        data = await self._http.execute(
            f"{TRACK_FIELDS} query LikedTracks {{ likedTracks {{ ...TrackFields }} }}"
        )
        return [Track.model_validate(t) for t in data.get("likedTracks", [])]

    async def like_track(self, id: str) -> None:
        await self._http.execute(
            "mutation LikeTrack($id: String!) { likeTrack(id: $id) }", {"id": id}
        )

    async def unlike_track(self, id: str) -> None:
        await self._http.execute(
            "mutation UnlikeTrack($id: String!) { unlikeTrack(id: $id) }", {"id": id}
        )

    # --- search ---------------------------------------------------------

    async def search(self, term: str) -> SearchResults:
        data = await self._http.execute(
            f"{TRACK_FIELDS} {ALBUM_FIELDS} {ARTIST_FIELDS} "
            "query Search($term: String!) { search(term: $term) { "
            "artists { ...ArtistFields } "
            "albums { ...AlbumFields } "
            "tracks { ...TrackFields } "
            "likedTracks { ...TrackFields } "
            "likedAlbums { ...AlbumFields } } }",
            {"term": term},
        )
        return SearchResults.model_validate(data.get("search") or {})

    # --- maintenance ----------------------------------------------------

    async def scan(self) -> None:
        """Trigger a library rescan."""
        await self._http.execute("mutation ScanLibrary { scanLibrary }")
