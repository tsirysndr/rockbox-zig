"""Domain API namespaces exposed on :class:`RockboxClient`."""

from .bluetooth import BluetoothApi
from .browse import BrowseApi
from .devices import DevicesApi
from .library import LibraryApi
from .playback import PlaybackApi
from .playlist import PlaylistApi
from .saved_playlists import CreatePlaylistInput, SavedPlaylistsApi, UpdatePlaylistInput
from .settings import SettingsApi
from .smart_playlists import (
    CreateSmartPlaylistInput,
    SmartPlaylistsApi,
    UpdateSmartPlaylistInput,
)
from .sound import SoundApi
from .system import SystemApi

__all__ = [
    "BluetoothApi",
    "BrowseApi",
    "CreatePlaylistInput",
    "CreateSmartPlaylistInput",
    "DevicesApi",
    "LibraryApi",
    "PlaybackApi",
    "PlaylistApi",
    "SavedPlaylistsApi",
    "SettingsApi",
    "SmartPlaylistsApi",
    "SoundApi",
    "SystemApi",
    "UpdatePlaylistInput",
    "UpdateSmartPlaylistInput",
]
