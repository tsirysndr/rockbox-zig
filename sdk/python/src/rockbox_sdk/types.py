"""Pydantic models and enums mirroring the Rockbox GraphQL schema.

The server replies in camelCase. Models accept both camelCase (the wire format) and
snake_case (the Python attribute names) thanks to ``populate_by_name=True`` plus
``alias_generator=to_camel``. That gives users full snake_case ergonomics on the
Python side without losing fidelity with the wire protocol.
"""

from __future__ import annotations

from enum import IntEnum
from typing import Any

from pydantic import BaseModel, ConfigDict
from pydantic.alias_generators import to_camel

# ---------------------------------------------------------------------------
# Enums — values match the firmware
# ---------------------------------------------------------------------------


class PlaybackStatus(IntEnum):
    STOPPED = 0
    PLAYING = 1
    PAUSED = 3


class RepeatMode(IntEnum):
    OFF = 0
    ALL = 1
    ONE = 2
    SHUFFLE = 3
    AB_REPEAT = 4


class ChannelConfig(IntEnum):
    STEREO = 0
    STEREO_NARROW = 1
    MONO = 2
    LEFT_MIX = 3
    RIGHT_MIX = 4
    KARAOKE = 5


class ReplaygainType(IntEnum):
    TRACK = 0
    ALBUM = 1
    SHUFFLE = 2


class InsertPosition(IntEnum):
    """Where to drop tracks when inserting into the queue (Kodi/Mopidy-style)."""

    NEXT = 0
    """After the currently playing track."""
    AFTER_CURRENT = 1
    """After the last manually inserted track."""
    LAST = 2
    """At the end of the playlist."""
    FIRST = 3
    """Replace the entire playlist."""


# ---------------------------------------------------------------------------
# Base model — camelCase wire ↔ snake_case Python
# ---------------------------------------------------------------------------


class _Model(BaseModel):
    model_config = ConfigDict(
        alias_generator=to_camel,
        populate_by_name=True,
        extra="ignore",
    )


# ---------------------------------------------------------------------------
# Audio types
# ---------------------------------------------------------------------------


class Track(_Model):
    id: str | None = None
    title: str = ""
    artist: str = ""
    album: str = ""
    genre: str = ""
    disc: str = ""
    track_string: str = ""
    year_string: str = ""
    composer: str = ""
    comment: str = ""
    album_artist: str = ""
    grouping: str = ""
    discnum: int = 0
    tracknum: int = 0
    layer: int = 0
    year: int = 0
    bitrate: int = 0
    frequency: int = 0
    filesize: int = 0
    length: int = 0
    """Duration in milliseconds."""
    elapsed: int = 0
    """Current playback position in milliseconds."""
    path: str = ""
    album_id: str | None = None
    artist_id: str | None = None
    genre_id: str | None = None
    album_art: str | None = None


class Album(_Model):
    id: str
    title: str
    artist: str
    year: int = 0
    year_string: str = ""
    album_art: str | None = None
    md5: str = ""
    artist_id: str = ""
    copyright_message: str | None = None
    tracks: list[Track] = []


class Artist(_Model):
    id: str
    name: str
    bio: str | None = None
    image: str | None = None
    tracks: list[Track] = []
    albums: list[Album] = []


class SearchResults(_Model):
    artists: list[Artist] = []
    albums: list[Album] = []
    tracks: list[Track] = []
    liked_tracks: list[Track] = []
    liked_albums: list[Album] = []


# ---------------------------------------------------------------------------
# Playlist types
# ---------------------------------------------------------------------------


class Playlist(_Model):
    amount: int
    index: int
    max_playlist_size: int = 0
    first_index: int = 0
    last_insert_pos: int = 0
    seed: int = 0
    last_shuffled_start: int = 0
    tracks: list[Track] = []


class SavedPlaylist(_Model):
    id: str
    name: str
    description: str | None = None
    image: str | None = None
    folder_id: str | None = None
    track_count: int = 0
    created_at: int = 0
    updated_at: int = 0


class SavedPlaylistFolder(_Model):
    id: str
    name: str
    created_at: int = 0
    updated_at: int = 0


class SmartPlaylist(_Model):
    id: str
    name: str
    description: str | None = None
    image: str | None = None
    folder_id: str | None = None
    is_system: bool = False
    rules: str = ""
    """JSON-encoded rules string."""
    created_at: int = 0
    updated_at: int = 0


class TrackStats(_Model):
    track_id: str
    play_count: int = 0
    skip_count: int = 0
    last_played: int | None = None
    last_skipped: int | None = None
    updated_at: int = 0


# ---------------------------------------------------------------------------
# Bluetooth, sound, devices, browse
# ---------------------------------------------------------------------------


class BluetoothDevice(_Model):
    address: str
    name: str
    paired: bool = False
    trusted: bool = False
    connected: bool = False
    rssi: int | None = None


class VolumeInfo(_Model):
    volume: int
    min: int
    max: int


class Device(_Model):
    id: str
    name: str
    host: str = ""
    ip: str = ""
    port: int = 0
    service: str = ""
    app: str = ""
    is_connected: bool = False
    base_url: str | None = None
    is_cast_device: bool = False
    is_source_device: bool = False
    is_current_device: bool = False


class Entry(_Model):
    name: str
    attr: int
    """Bitmask: bit 4 (0x10) = directory."""
    time_write: int = 0
    customaction: int = 0
    display_name: str | None = None

    @property
    def is_directory(self) -> bool:
        return (self.attr & 0x10) != 0


def is_directory(entry: Entry) -> bool:
    """Mirror of the TypeScript ``isDirectory`` helper."""
    return entry.is_directory


# ---------------------------------------------------------------------------
# System & settings
# ---------------------------------------------------------------------------


class SystemStatus(_Model):
    resume_index: int = 0
    resume_crc32: int = 0
    resume_elapsed: int = 0
    resume_offset: int = 0
    runtime: int = 0
    topruntime: int = 0
    dircache_size: int = 0
    last_screen: int = 0
    viewer_icon_count: int = 0
    last_volume_change: int = 0


class EqBandSetting(_Model):
    cutoff: int
    q: int
    gain: int


class ReplaygainSettings(_Model):
    noclip: bool = False
    type: int = 0
    preamp: int = 0


class CompressorSettings(_Model):
    threshold: int = 0
    makeup_gain: int = 0
    ratio: int = 0
    knee: int = 0
    release_time: int = 0
    attack_time: int = 0


class UserSettings(_Model):
    """Server-side settings as returned by ``globalSettings``.

    Tolerant of new fields the server might add — anything unknown is dropped.
    """

    music_dir: str = ""
    volume: int = 0
    balance: int = 0
    bass: int = 0
    treble: int = 0
    channel_config: int = 0
    stereo_width: int = 0
    eq_enabled: bool = False
    eq_precut: int = 0
    eq_band_settings: list[EqBandSetting] = []
    replaygain_settings: ReplaygainSettings = ReplaygainSettings()
    compressor_settings: CompressorSettings = CompressorSettings()
    crossfade_enabled: int = 0
    crossfade_fade_in_delay: int = 0
    crossfade_fade_in_duration: int = 0
    crossfade_fade_out_delay: int = 0
    crossfade_fade_out_duration: int = 0
    crossfade_fade_out_mixmode: int = 0
    crossfeed_enabled: bool = False
    crossfeed_direct_gain: int = 0
    crossfeed_cross_gain: int = 0
    crossfeed_hf_attenuation: int = 0
    crossfeed_hf_cutoff: int = 0
    repeat_mode: int = 0
    single_mode: bool = False
    party_mode: bool = False
    shuffle: bool = False
    player_name: str = ""


# Used as input to settings.save() — accepts any partial subset of UserSettings
PartialUserSettings = dict[str, Any]
