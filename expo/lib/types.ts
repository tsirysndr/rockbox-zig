export type Track = {
  id: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  artwork?: string;
  liked?: boolean;
};

export type Album = {
  id: string;
  title: string;
  artist: string;
  artwork: string;
  year?: number;
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
