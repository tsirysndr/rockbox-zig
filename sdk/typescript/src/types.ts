// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

export enum PlaybackStatus {
  Stopped = 0,
  Playing = 1,
  Paused = 2,
}

export enum RepeatMode {
  Off = 0,
  All = 1,
  One = 2,
  Shuffle = 3,
  ABRepeat = 4,
}

export enum ChannelConfig {
  Stereo = 0,
  StereoNarrow = 1,
  Mono = 2,
  LeftMix = 3,
  RightMix = 4,
  Karaoke = 5,
}

export enum ReplaygainType {
  Track = 0,
  Album = 1,
  Shuffle = 2,
}

// ---------------------------------------------------------------------------
// Core audio types
// ---------------------------------------------------------------------------

export interface Track {
  id?: string;
  title: string;
  artist: string;
  album: string;
  genre: string;
  disc: string;
  trackString: string;
  yearString: string;
  composer: string;
  comment: string;
  albumArtist: string;
  grouping: string;
  discnum: number;
  tracknum: number;
  layer: number;
  year: number;
  bitrate: number;
  frequency: number;
  filesize: number;
  /** Duration in milliseconds */
  length: number;
  /** Current playback position in milliseconds */
  elapsed: number;
  path: string;
  albumId?: string;
  artistId?: string;
  genreId?: string;
  albumArt?: string;
}

export interface Album {
  id: string;
  title: string;
  artist: string;
  year: number;
  yearString: string;
  albumArt?: string;
  md5: string;
  artistId: string;
  tracks: Track[];
}

export interface Artist {
  id: string;
  name: string;
  bio?: string;
  image?: string;
  tracks: Track[];
  albums: Album[];
}

export interface SearchResults {
  artists: Artist[];
  albums: Album[];
  tracks: Track[];
  likedTracks: Track[];
  likedAlbums: Album[];
}

// ---------------------------------------------------------------------------
// Playlist types
// ---------------------------------------------------------------------------

export interface Playlist {
  amount: number;
  index: number;
  maxPlaylistSize: number;
  firstIndex: number;
  lastInsertPos: number;
  seed: number;
  lastShuffledStart: number;
  tracks: Track[];
}

export interface SavedPlaylist {
  id: string;
  name: string;
  description?: string;
  image?: string;
  folderId?: string;
  trackCount: number;
  createdAt: number;
  updatedAt: number;
}

export interface SavedPlaylistFolder {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
}

export interface SmartPlaylist {
  id: string;
  name: string;
  description?: string;
  image?: string;
  folderId?: string;
  isSystem: boolean;
  /** JSON-encoded rules */
  rules: string;
  createdAt: number;
  updatedAt: number;
}

export interface TrackStats {
  trackId: string;
  playCount: number;
  skipCount: number;
  lastPlayed?: number;
  lastSkipped?: number;
  updatedAt: number;
}

// ---------------------------------------------------------------------------
// Device types
// ---------------------------------------------------------------------------

export interface Device {
  id: string;
  name: string;
  host: string;
  ip: string;
  port: number;
  service: string;
  app: string;
  isConnected: boolean;
  baseUrl?: string;
  isCastDevice: boolean;
  isSourceDevice: boolean;
  isCurrentDevice: boolean;
}

// ---------------------------------------------------------------------------
// File browser types
// ---------------------------------------------------------------------------

export interface Entry {
  name: string;
  /** Bitmask: bit 4 = directory */
  attr: number;
  timeWrite: number;
  customaction: number;
}

export function isDirectory(entry: Entry): boolean {
  return (entry.attr & 0x10) !== 0;
}

// ---------------------------------------------------------------------------
// System types
// ---------------------------------------------------------------------------

export interface SystemStatus {
  resumeIndex: number;
  resumeCrc32: number;
  resumeElapsed: number;
  resumeOffset: number;
  runtime: number;
  topruntime: number;
  dircacheSize: number;
  lastScreen: number;
  viewerIconCount: number;
  lastVolumeChange: number;
}

// ---------------------------------------------------------------------------
// Settings types
// ---------------------------------------------------------------------------

export interface EqBandSetting {
  cutoff: number;
  q: number;
  gain: number;
}

export interface ReplaygainSettings {
  noclip: boolean;
  type: number;
  preamp: number;
}

export interface CompressorSettings {
  threshold: number;
  makeupGain: number;
  ratio: number;
  knee: number;
  releaseTime: number;
  attackTime: number;
}

export interface UserSettings {
  musicDir: string;
  volume: number;
  balance: number;
  bass: number;
  treble: number;
  channelConfig: number;
  stereoWidth: number;
  eqEnabled: boolean;
  eqPrecut: number;
  eqBandSettings: EqBandSetting[];
  replaygainSettings: ReplaygainSettings;
  compressorSettings: CompressorSettings;
  crossfadeEnabled: number;
  crossfadeFadeInDelay: number;
  crossfadeFadeInDuration: number;
  crossfadeFadeOutDelay: number;
  crossfadeFadeOutDuration: number;
  crossfadeFadeOutMixmode: number;
  crossfeedEnabled: boolean;
  crossfeedDirectGain: number;
  crossfeedCrossGain: number;
  crossfeedHfAttenuation: number;
  crossfeedHfCutoff: number;
  repeatMode: number;
  singleMode: boolean;
  partyMode: boolean;
  shuffle: boolean;
  playerName: string;
  [key: string]: unknown;
}

export type PartialUserSettings = Partial<Omit<UserSettings, 'eqBandSettings' | 'replaygainSettings' | 'compressorSettings'>> & {
  eqBandSettings?: EqBandSetting[];
  replaygainSettings?: ReplaygainSettings;
  compressorSettings?: CompressorSettings;
};

// ---------------------------------------------------------------------------
// Insert position constants (Kodi / Mopidy convention)
// ---------------------------------------------------------------------------

export const InsertPosition = {
  /** After the currently playing track */
  Next: 0,
  /** After the last manually inserted track */
  AfterCurrent: 1,
  /** At the end of the playlist */
  Last: 2,
  /** Replace the entire playlist */
  First: 3,
} as const;
export type InsertPosition = (typeof InsertPosition)[keyof typeof InsertPosition];
