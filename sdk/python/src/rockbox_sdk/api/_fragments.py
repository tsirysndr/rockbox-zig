"""Reusable GraphQL fragments. Inlined verbatim into queries — keep in sync with the schema."""

TRACK_FIELDS = """
fragment TrackFields on Track {
  id title artist album genre disc trackString yearString
  composer comment albumArtist grouping
  discnum tracknum layer year bitrate frequency
  filesize length elapsed path
  albumId artistId genreId albumArt
}
"""

ALBUM_FIELDS = """
fragment AlbumFields on Album {
  id title artist year yearString albumArt md5 artistId copyrightMessage
}
"""

ARTIST_FIELDS = """
fragment ArtistFields on Artist {
  id name bio image
}
"""

BLUETOOTH_DEVICE_FIELDS = """
fragment BluetoothDeviceFields on BluetoothDevice {
  address name paired trusted connected rssi
}
"""
