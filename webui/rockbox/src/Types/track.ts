export type Track = {
  id: string;
  trackNumber?: number | null;
  title: string;
  path?: string;
  artist: string;
  album?: string | null;
  time?: string | null;
  duration?: number | null;
  albumArt?: string | null;
  cover?: string | null;
  albumId?: string | null;
  artistId?: string | null;
  discnum?: number | null;
};

export type CurrentTrack = {
  id?: string;
  album?: string;
  artist?: string;
  title?: string;
  cover?: string;
  duration: number;
  progress: number;
  isPlaying?: boolean;
  albumId?: string | null;
  artistId?: string | null;
};
