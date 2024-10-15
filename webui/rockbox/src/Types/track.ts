export type Track = {
  id: string;
  trackNumber?: number;
  title: string;
  artist: string;
  album?: string;
  time?: string;
  duration?: number;
  albumArt?: string;
  cover?: string;
  albumId?: string;
  artistId?: string;
  discnum?: number;
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
