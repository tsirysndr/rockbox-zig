export type Track = {
  id: string;
  /** Filesystem path on the daemon — required by `playTrack` / `insertTrack`. */
  path?: string;
  title: string;
  artist: string;
  /** Empty when the proto doesn't tell us the linked artist row. */
  artistId?: string;
  album: string;
  /** Empty when the proto doesn't tell us the linked album row. */
  albumId?: string;
  duration: number;
  artwork?: string;
  liked?: boolean;
  trackNumber?: number;
  discNumber?: number;
};

export type Album = {
  id: string;
  title: string;
  artist: string;
  artwork: string;
  year?: number;
  /** Raw release date string from the daemon, e.g. "2014-12-09". */
  yearString?: string;
  copyrightMessage?: string;
  genre?: string;
};

export type Genre = {
  id: string;
  name: string;
  color: string;
};

export type Artist = {
  id: string;
  name: string;
  image: string;
  followers?: string;
};

export type Playlist = {
  id: string;
  name: string;
  description?: string;
  artwork: string;
  trackCount: number;
  isSmart?: boolean;
  rules?: string;
};

export type LibrarySection =
  | "songs"
  | "albums"
  | "artists"
  | "playlists"
  | "liked";

export type RepeatMode = "off" | "all" | "one";

export type FileEntry = {
  name: string;
  path: string;
  is_dir: boolean;
};

export type FilesMode =
  | "root"
  | "local"
  | "upnp-devices"
  | "upnp-browse"
  | "plex-servers"
  | "plex-browse"
  | "jellyfin-servers"
  | "jellyfin-browse"
  | "navidrome-browse"
  | "kodi-servers"
  | "kodi-browse";
