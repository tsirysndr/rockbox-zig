"""Tests for the top-level client and HTTP transport.

Uses ``pytest-httpx`` to mock the GraphQL endpoint — no live rockboxd needed.
"""

from __future__ import annotations

import pytest
from pytest_httpx import HTTPXMock

from rockbox_sdk import (
    InsertPosition,
    PlaybackStatus,
    RockboxClient,
    RockboxGraphQLError,
    RockboxNetworkError,
)


def _ok(data: dict) -> dict:
    return {"data": data}


@pytest.mark.asyncio
async def test_builder_sets_urls() -> None:
    client = RockboxClient.builder().host("nas.local").port(1234).build()
    assert client._config.resolve_http_url() == "http://nas.local:1234/graphql"
    assert client._config.resolve_ws_url() == "ws://nas.local:1234/graphql"
    await client.aclose()


@pytest.mark.asyncio
async def test_full_url_overrides(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(json=_ok({"status": 1}), url="http://override/graphql")
    async with RockboxClient(http_url="http://override/graphql") as client:
        assert await client.playback.status() == PlaybackStatus.PLAYING


@pytest.mark.asyncio
async def test_status_typed(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(json=_ok({"status": 3}))
    async with RockboxClient() as client:
        assert await client.playback.status() == PlaybackStatus.PAUSED


@pytest.mark.asyncio
async def test_current_track_returns_pydantic_model(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(
        json=_ok(
            {
                "currentTrack": {
                    "title": "Money",
                    "artist": "Pink Floyd",
                    "album": "The Dark Side of the Moon",
                    "albumArt": "http://x/art.jpg",
                    "albumId": "a-1",
                    "length": 382000,
                    "elapsed": 90000,
                }
            }
        )
    )
    async with RockboxClient() as client:
        track = await client.playback.current_track()
        assert track is not None
        assert track.title == "Money"
        # Snake_case Python access for camelCase wire fields
        assert track.album_art == "http://x/art.jpg"
        assert track.album_id == "a-1"
        assert track.length == 382000


@pytest.mark.asyncio
async def test_search_results_parse(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(
        json=_ok(
            {
                "search": {
                    "artists": [],
                    "albums": [
                        {
                            "id": "1",
                            "title": "X",
                            "artist": "Y",
                            "year": 1973,
                            "yearString": "1973",
                            "albumArt": None,
                            "md5": "deadbeef",
                            "artistId": "a",
                            "copyrightMessage": None,
                        }
                    ],
                    "tracks": [],
                    "likedTracks": [],
                    "likedAlbums": [],
                }
            }
        )
    )
    async with RockboxClient() as client:
        results = await client.library.search("x")
        assert len(results.albums) == 1
        assert results.albums[0].year == 1973


@pytest.mark.asyncio
async def test_graphql_error_raises(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(
        json={"data": None, "errors": [{"message": "boom"}]}
    )
    async with RockboxClient() as client:
        with pytest.raises(RockboxGraphQLError) as excinfo:
            await client.playback.status()
        assert "boom" in str(excinfo.value)


@pytest.mark.asyncio
async def test_network_error_raises(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(status_code=502)
    async with RockboxClient() as client:
        with pytest.raises(RockboxNetworkError):
            await client.playback.status()


@pytest.mark.asyncio
async def test_insert_position_serialised_as_int(httpx_mock: HTTPXMock) -> None:
    httpx_mock.add_response(json=_ok({"insertTracks": True}))
    async with RockboxClient() as client:
        await client.playlist.insert_tracks(["/a.mp3"], InsertPosition.LAST)

    request = httpx_mock.get_request()
    assert request is not None
    body = request.read().decode()
    assert '"position":2' in body or '"position": 2' in body
