"""Python SDK for Rockbox.

Quick start::

    import asyncio
    from rockbox_sdk import RockboxClient, PlaybackStatus

    async def main():
        async with RockboxClient(host="localhost") as client:
            track = await client.playback.current_track()
            if track:
                print(f"Now: {track.title} — {track.artist}")
            await client.playback.play()

    asyncio.run(main())
"""

from __future__ import annotations

from .api import (
    BluetoothApi,
    BrowseApi,
    CreatePlaylistInput,
    CreateSmartPlaylistInput,
    DevicesApi,
    LibraryApi,
    PlaybackApi,
    PlaylistApi,
    SavedPlaylistsApi,
    SettingsApi,
    SmartPlaylistsApi,
    SoundApi,
    SystemApi,
    UpdatePlaylistInput,
    UpdateSmartPlaylistInput,
)
from .client import RockboxClient, RockboxClientBuilder, RockboxClientConfig
from .errors import RockboxError, RockboxGraphQLError, RockboxNetworkError
from .events import (
    PLAYLIST_CHANGED,
    STATUS_CHANGED,
    TRACK_CHANGED,
    WS_CLOSE,
    WS_ERROR,
    WS_OPEN,
    EventEmitter,
)
from .plugin import PluginContext, PluginRegistry, RockboxPlugin
from .types import (
    Album,
    Artist,
    BluetoothDevice,
    ChannelConfig,
    CompressorSettings,
    Device,
    Entry,
    EqBandSetting,
    InsertPosition,
    PartialUserSettings,
    PlaybackStatus,
    Playlist,
    RepeatMode,
    ReplaygainSettings,
    ReplaygainType,
    SavedPlaylist,
    SavedPlaylistFolder,
    SearchResults,
    SmartPlaylist,
    SystemStatus,
    Track,
    TrackStats,
    UserSettings,
    VolumeInfo,
    is_directory,
)

__version__ = "0.1.0"

__all__ = [
    # Client
    "RockboxClient",
    "RockboxClientBuilder",
    "RockboxClientConfig",
    # APIs (re-exported in case users want to mock them)
    "BluetoothApi",
    "BrowseApi",
    "DevicesApi",
    "LibraryApi",
    "PlaybackApi",
    "PlaylistApi",
    "SavedPlaylistsApi",
    "SettingsApi",
    "SmartPlaylistsApi",
    "SoundApi",
    "SystemApi",
    # Inputs
    "CreatePlaylistInput",
    "CreateSmartPlaylistInput",
    "UpdatePlaylistInput",
    "UpdateSmartPlaylistInput",
    # Errors
    "RockboxError",
    "RockboxGraphQLError",
    "RockboxNetworkError",
    # Plugin system
    "PluginContext",
    "PluginRegistry",
    "RockboxPlugin",
    # Events
    "EventEmitter",
    "TRACK_CHANGED",
    "STATUS_CHANGED",
    "PLAYLIST_CHANGED",
    "WS_OPEN",
    "WS_CLOSE",
    "WS_ERROR",
    # Enums
    "ChannelConfig",
    "InsertPosition",
    "PlaybackStatus",
    "RepeatMode",
    "ReplaygainType",
    # Models
    "Album",
    "Artist",
    "BluetoothDevice",
    "CompressorSettings",
    "Device",
    "Entry",
    "EqBandSetting",
    "PartialUserSettings",
    "Playlist",
    "ReplaygainSettings",
    "SavedPlaylist",
    "SavedPlaylistFolder",
    "SearchResults",
    "SmartPlaylist",
    "SystemStatus",
    "Track",
    "TrackStats",
    "UserSettings",
    "VolumeInfo",
    # Helpers
    "is_directory",
    "__version__",
]
