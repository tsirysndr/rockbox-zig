import { gql } from '@apollo/client';
import * as Apollo from '@apollo/client';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
const defaultOptions = {} as const;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
};

export type Album = {
  __typename?: 'Album';
  albumArt?: Maybe<Scalars['String']['output']>;
  artist: Scalars['String']['output'];
  artistId: Scalars['String']['output'];
  id: Scalars['String']['output'];
  md5: Scalars['String']['output'];
  title: Scalars['String']['output'];
  tracks: Array<Track>;
  year: Scalars['Int']['output'];
  yearString: Scalars['String']['output'];
};

export type Artist = {
  __typename?: 'Artist';
  albums: Array<Album>;
  bio?: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  image?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  tracks: Array<Track>;
};

export type AudioStatus = {
  __typename?: 'AudioStatus';
  status: Scalars['Int']['output'];
};

export type CompressorSettings = {
  __typename?: 'CompressorSettings';
  attackTime: Scalars['Int']['output'];
  knee: Scalars['Int']['output'];
  makeupGain: Scalars['Int']['output'];
  ratio: Scalars['Int']['output'];
  releaseTime: Scalars['Int']['output'];
  threshold: Scalars['Int']['output'];
};

export type Entry = {
  __typename?: 'Entry';
  attr: Scalars['Int']['output'];
  customaction: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  timeWrite: Scalars['Int']['output'];
};

export type EqBandSetting = {
  __typename?: 'EqBandSetting';
  cutoff: Scalars['Int']['output'];
  gain: Scalars['Int']['output'];
  q: Scalars['Int']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  adjustVolume: Scalars['Int']['output'];
  beepPlay: Scalars['String']['output'];
  fastForwardRewind: Scalars['Int']['output'];
  flushAndReloadTracks: Scalars['Int']['output'];
  hardStop: Scalars['Int']['output'];
  insertAlbum: Scalars['Int']['output'];
  insertDirectory: Scalars['Int']['output'];
  insertPlaylist: Scalars['String']['output'];
  insertTracks: Scalars['Int']['output'];
  keyclickClick: Scalars['String']['output'];
  likeAlbum: Scalars['Int']['output'];
  likeTrack: Scalars['Int']['output'];
  next: Scalars['Int']['output'];
  pause: Scalars['Int']['output'];
  pcmbufFade: Scalars['String']['output'];
  pcmbufPlay: Scalars['String']['output'];
  pcmbufSetLowLatency: Scalars['String']['output'];
  play: Scalars['Int']['output'];
  playAlbum: Scalars['Int']['output'];
  playArtistTracks: Scalars['Int']['output'];
  playDirectory: Scalars['Int']['output'];
  playPlaylist: Scalars['Int']['output'];
  playTrack: Scalars['Int']['output'];
  playlistCreate: Scalars['Int']['output'];
  playlistRemoveAllTracks: Scalars['Int']['output'];
  playlistRemoveTrack: Scalars['Int']['output'];
  playlistResume: Scalars['Int']['output'];
  playlistSetModified: Scalars['String']['output'];
  playlistStart: Scalars['Int']['output'];
  playlistSync: Scalars['String']['output'];
  previous: Scalars['Int']['output'];
  resume: Scalars['Int']['output'];
  resumeTrack: Scalars['String']['output'];
  setPitch: Scalars['String']['output'];
  shufflePlaylist: Scalars['Int']['output'];
  soundMax: Scalars['String']['output'];
  soundMin: Scalars['String']['output'];
  soundSet: Scalars['String']['output'];
  soundUnit: Scalars['String']['output'];
  systemSoundPlay: Scalars['String']['output'];
  unlikeAlbum: Scalars['Int']['output'];
  unlikeTrack: Scalars['Int']['output'];
};


export type MutationAdjustVolumeArgs = {
  steps: Scalars['Int']['input'];
};


export type MutationFastForwardRewindArgs = {
  newTime: Scalars['Int']['input'];
};


export type MutationInsertAlbumArgs = {
  albumId: Scalars['String']['input'];
  position: Scalars['Int']['input'];
};


export type MutationInsertDirectoryArgs = {
  directory: Scalars['String']['input'];
  playlistId?: InputMaybe<Scalars['String']['input']>;
  position: Scalars['Int']['input'];
};


export type MutationInsertPlaylistArgs = {
  playlistId: Scalars['String']['input'];
  position: Scalars['Int']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
  targetPlaylistId?: InputMaybe<Scalars['String']['input']>;
};


export type MutationInsertTracksArgs = {
  playlistId?: InputMaybe<Scalars['String']['input']>;
  position: Scalars['Int']['input'];
  tracks: Array<Scalars['String']['input']>;
};


export type MutationLikeAlbumArgs = {
  id: Scalars['String']['input'];
};


export type MutationLikeTrackArgs = {
  id: Scalars['String']['input'];
};


export type MutationPlayArgs = {
  elapsed: Scalars['Int']['input'];
  offset: Scalars['Int']['input'];
};


export type MutationPlayAlbumArgs = {
  albumId: Scalars['String']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayArtistTracksArgs = {
  artistId: Scalars['String']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayDirectoryArgs = {
  path: Scalars['String']['input'];
  recurse?: InputMaybe<Scalars['Boolean']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayPlaylistArgs = {
  playlistId: Scalars['String']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayTrackArgs = {
  path: Scalars['String']['input'];
};


export type MutationPlaylistCreateArgs = {
  name: Scalars['String']['input'];
  tracks: Array<Scalars['String']['input']>;
};


export type MutationPlaylistRemoveTrackArgs = {
  index: Scalars['Int']['input'];
};


export type MutationPlaylistStartArgs = {
  elapsed?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  startIndex?: InputMaybe<Scalars['Int']['input']>;
};


export type MutationUnlikeAlbumArgs = {
  id: Scalars['String']['input'];
};


export type MutationUnlikeTrackArgs = {
  id: Scalars['String']['input'];
};

export type Playlist = {
  __typename?: 'Playlist';
  amount: Scalars['Int']['output'];
  firstIndex: Scalars['Int']['output'];
  index: Scalars['Int']['output'];
  lastInsertPos: Scalars['Int']['output'];
  lastShuffledStart: Scalars['Int']['output'];
  maxPlaylistSize: Scalars['Int']['output'];
  seed: Scalars['Int']['output'];
  tracks: Array<Track>;
};

export type Query = {
  __typename?: 'Query';
  album?: Maybe<Album>;
  albums: Array<Album>;
  artist?: Maybe<Artist>;
  artists: Array<Artist>;
  currentTrack?: Maybe<Track>;
  getDisplayIndex: Scalars['String']['output'];
  getFilePosition: Scalars['Int']['output'];
  getFirstIndex: Scalars['String']['output'];
  getPitch: Scalars['String']['output'];
  getResumeInfo: Scalars['String']['output'];
  getTrackInfo: Scalars['String']['output'];
  globalSettings: UserSettings;
  globalStatus: SystemStatus;
  likedAlbums: Array<Album>;
  likedTracks: Array<Track>;
  nextTrack?: Maybe<Track>;
  playlistAmount: Scalars['Int']['output'];
  playlistGetCurrent: Playlist;
  rockboxVersion: Scalars['String']['output'];
  soundCurrent: Scalars['String']['output'];
  soundDefault: Scalars['String']['output'];
  soundVal2Phys: Scalars['String']['output'];
  status: Scalars['Int']['output'];
  track?: Maybe<Track>;
  tracks: Array<Track>;
  treeGetEntries: Array<Entry>;
};


export type QueryAlbumArgs = {
  id: Scalars['String']['input'];
};


export type QueryArtistArgs = {
  id: Scalars['String']['input'];
};


export type QueryTrackArgs = {
  id: Scalars['String']['input'];
};


export type QueryTreeGetEntriesArgs = {
  path?: InputMaybe<Scalars['String']['input']>;
};

export type ReplaygainSettings = {
  __typename?: 'ReplaygainSettings';
  noclip: Scalars['Boolean']['output'];
  preamp: Scalars['Int']['output'];
  type: Scalars['Int']['output'];
};

export type Subscription = {
  __typename?: 'Subscription';
  currentlyPlayingSong: Track;
  playbackStatus: AudioStatus;
  playlistChanged: Playlist;
};

export type SystemStatus = {
  __typename?: 'SystemStatus';
  dircacheSize: Scalars['Int']['output'];
  lastScreen: Scalars['Int']['output'];
  lastVolumeChange: Scalars['Int']['output'];
  resumeCrc32: Scalars['Int']['output'];
  resumeElapsed: Scalars['Int']['output'];
  resumeIndex: Scalars['Int']['output'];
  resumeOffset: Scalars['Int']['output'];
  runtime: Scalars['Int']['output'];
  topruntime: Scalars['Int']['output'];
  viewerIconCount: Scalars['Int']['output'];
};

export type Track = {
  __typename?: 'Track';
  album: Scalars['String']['output'];
  albumArt?: Maybe<Scalars['String']['output']>;
  albumArtist: Scalars['String']['output'];
  albumId?: Maybe<Scalars['String']['output']>;
  artist: Scalars['String']['output'];
  artistId?: Maybe<Scalars['String']['output']>;
  bitrate: Scalars['Int']['output'];
  comment: Scalars['String']['output'];
  composer: Scalars['String']['output'];
  disc: Scalars['String']['output'];
  discnum: Scalars['Int']['output'];
  elapsed: Scalars['Int']['output'];
  filesize: Scalars['Int']['output'];
  frequency: Scalars['Int']['output'];
  genre: Scalars['String']['output'];
  genreId?: Maybe<Scalars['String']['output']>;
  grouping: Scalars['String']['output'];
  id?: Maybe<Scalars['String']['output']>;
  layer: Scalars['Int']['output'];
  length: Scalars['Int']['output'];
  path: Scalars['String']['output'];
  title: Scalars['String']['output'];
  trackString: Scalars['String']['output'];
  tracknum: Scalars['Int']['output'];
  year: Scalars['Int']['output'];
  yearString: Scalars['String']['output'];
};

export type UserSettings = {
  __typename?: 'UserSettings';
  afrEnabled: Scalars['Int']['output'];
  albumArt: Scalars['Int']['output'];
  autocreatebookmark: Scalars['Int']['output'];
  autoloadbookmark: Scalars['Int']['output'];
  autoresumeAutomatic: Scalars['Int']['output'];
  autoresumeEnable: Scalars['Boolean']['output'];
  autoresumePaths: Scalars['String']['output'];
  autoupdatebookmark: Scalars['Boolean']['output'];
  backdropFile: Scalars['String']['output'];
  backlightOnButtonHold: Scalars['Int']['output'];
  backlightTimeout: Scalars['Int']['output'];
  backlightTimeoutPlugged: Scalars['Int']['output'];
  balance: Scalars['Int']['output'];
  bass: Scalars['Int']['output'];
  bassCutoff: Scalars['Int']['output'];
  batteryCapacity: Scalars['Int']['output'];
  batteryDisplay: Scalars['Int']['output'];
  batteryType: Scalars['Int']['output'];
  beep: Scalars['Int']['output'];
  bgColor: Scalars['Int']['output'];
  bidirLimit: Scalars['Int']['output'];
  blFilterFirstKeypress: Scalars['Boolean']['output'];
  blSelectiveActions: Scalars['Boolean']['output'];
  blSelectiveActionsMask: Scalars['Int']['output'];
  brightness: Scalars['Int']['output'];
  browseCurrent: Scalars['Boolean']['output'];
  browserDefault: Scalars['Int']['output'];
  btSelectiveSoftlockActions: Scalars['Boolean']['output'];
  btSelectiveSoftlockActionsMask: Scalars['Int']['output'];
  bufferMargin: Scalars['Int']['output'];
  captionBacklight: Scalars['Boolean']['output'];
  carAdapterMode: Scalars['Boolean']['output'];
  carAdapterModeDelay: Scalars['Int']['output'];
  channelConfig: Scalars['Int']['output'];
  colorsFile: Scalars['String']['output'];
  compressorSettings: CompressorSettings;
  constrainNextFolder: Scalars['Boolean']['output'];
  contrast: Scalars['Int']['output'];
  crossfade: Scalars['Int']['output'];
  crossfadeFadeInDelay: Scalars['Int']['output'];
  crossfadeFadeInDuration: Scalars['Int']['output'];
  crossfadeFadeOutDelay: Scalars['Int']['output'];
  crossfadeFadeOutDuration: Scalars['Int']['output'];
  crossfadeFadeOutMixmode: Scalars['Int']['output'];
  crossfeed: Scalars['Int']['output'];
  crossfeedCrossGain: Scalars['Int']['output'];
  crossfeedDirectGain: Scalars['Int']['output'];
  crossfeedHfAttenuation: Scalars['Int']['output'];
  crossfeedHfCutoff: Scalars['Int']['output'];
  cuesheet: Scalars['Boolean']['output'];
  cursorStyle: Scalars['Int']['output'];
  defaultCodepage: Scalars['Int']['output'];
  depth3D: Scalars['Int']['output'];
  dircache: Scalars['Boolean']['output'];
  dirfilter: Scalars['Int']['output'];
  disableMainmenuScrolling: Scalars['Boolean']['output'];
  diskSpindown: Scalars['Int']['output'];
  ditheringEnabled: Scalars['Boolean']['output'];
  eqBandSettings: Array<EqBandSetting>;
  eqEnabled: Scalars['Boolean']['output'];
  eqPrecut: Scalars['Int']['output'];
  fadeOnStop: Scalars['Boolean']['output'];
  ffRewindAccel: Scalars['Int']['output'];
  ffRewindMinStep: Scalars['Int']['output'];
  fgColor: Scalars['Int']['output'];
  flipDisplay: Scalars['Boolean']['output'];
  fontFile: Scalars['String']['output'];
  glyphsToCache: Scalars['Int']['output'];
  governor: Scalars['Int']['output'];
  holdLrForScrollInList: Scalars['Boolean']['output'];
  hotkeyTree: Scalars['Int']['output'];
  hotkeyWps: Scalars['Int']['output'];
  iconFile: Scalars['String']['output'];
  interpretNumbers: Scalars['Int']['output'];
  invert: Scalars['Boolean']['output'];
  kbdFile: Scalars['String']['output'];
  keepCurrentTrackOnReplacePlaylist: Scalars['Boolean']['output'];
  keyclick: Scalars['Int']['output'];
  keyclickHardware: Scalars['Boolean']['output'];
  keyclickRepeats: Scalars['Int']['output'];
  keypressRestartsSleeptimer: Scalars['Boolean']['output'];
  langFile: Scalars['String']['output'];
  lcdSleepAfterBacklightOff: Scalars['Int']['output'];
  listAccelStartDelay: Scalars['Int']['output'];
  listAccelWait: Scalars['Int']['output'];
  listLinePadding: Scalars['Int']['output'];
  listOrder: Scalars['Int']['output'];
  listSeparatorColor: Scalars['Int']['output'];
  listSeparatorHeight: Scalars['Int']['output'];
  listWraparound: Scalars['Boolean']['output'];
  lseColor: Scalars['Int']['output'];
  lssColor: Scalars['Int']['output'];
  lstColor: Scalars['Int']['output'];
  maxFilesInDir: Scalars['Int']['output'];
  maxFilesInPlaylist: Scalars['Int']['output'];
  nextFolder: Scalars['Int']['output'];
  offsetOutOfView: Scalars['Boolean']['output'];
  partyMode: Scalars['Boolean']['output'];
  pauseRewind: Scalars['Int']['output'];
  pbe: Scalars['Int']['output'];
  pbePrecut: Scalars['Int']['output'];
  peakMeterClipHold: Scalars['Int']['output'];
  peakMeterDbfs: Scalars['Boolean']['output'];
  peakMeterHold: Scalars['Int']['output'];
  peakMeterMax: Scalars['Int']['output'];
  peakMeterMin: Scalars['Int']['output'];
  peakMeterRelease: Scalars['Int']['output'];
  pitchModeSemitone: Scalars['Boolean']['output'];
  pitchModeTimestretch: Scalars['Boolean']['output'];
  playFrequency: Scalars['Int']['output'];
  playSelected: Scalars['Boolean']['output'];
  playerName: Scalars['String']['output'];
  playlistCatalogDir: Scalars['String']['output'];
  playlistShuffle: Scalars['Boolean']['output'];
  playlistViewerIcons: Scalars['Boolean']['output'];
  playlistViewerIndices: Scalars['Boolean']['output'];
  playlistViewerTrackDisplay: Scalars['Int']['output'];
  powerMode: Scalars['Int']['output'];
  poweroff: Scalars['Int']['output'];
  preventSkip: Scalars['Boolean']['output'];
  recursiveDirInsert: Scalars['Int']['output'];
  repeatMode: Scalars['Int']['output'];
  replaygainSettings: ReplaygainSettings;
  resumeRewind: Scalars['Int']['output'];
  rewindAcrossTracks: Scalars['Boolean']['output'];
  rollOff: Scalars['Int']['output'];
  rootMenuCustomized: Scalars['Boolean']['output'];
  runtimedb: Scalars['Boolean']['output'];
  sbsFile: Scalars['String']['output'];
  screenScrollStep: Scalars['Int']['output'];
  scrollDelay: Scalars['Int']['output'];
  scrollPaginated: Scalars['Boolean']['output'];
  scrollSpeed: Scalars['Int']['output'];
  scrollStep: Scalars['Int']['output'];
  scrollbar: Scalars['Int']['output'];
  scrollbarWidth: Scalars['Int']['output'];
  shortcutsReplacesQs: Scalars['Boolean']['output'];
  showFilenameExt: Scalars['Int']['output'];
  showIcons: Scalars['Boolean']['output'];
  showPathInBrowser: Scalars['Int']['output'];
  showQueueOptions: Scalars['Int']['output'];
  showShuffledAddingOptions: Scalars['Boolean']['output'];
  showShutdownMessage: Scalars['Boolean']['output'];
  singleMode: Scalars['Int']['output'];
  skipLength: Scalars['Int']['output'];
  sleeptimerDuration: Scalars['Int']['output'];
  sleeptimerOnStartup: Scalars['Boolean']['output'];
  sortCase: Scalars['Boolean']['output'];
  sortDir: Scalars['Int']['output'];
  sortFile: Scalars['Int']['output'];
  spdifEnable: Scalars['Boolean']['output'];
  speakerMode: Scalars['Int']['output'];
  startDirectory: Scalars['String']['output'];
  startInScreen: Scalars['Int']['output'];
  statusbar: Scalars['Int']['output'];
  stereoWidth: Scalars['Int']['output'];
  stereoswMode: Scalars['Int']['output'];
  surroundBalance: Scalars['Int']['output'];
  surroundEnabled: Scalars['Int']['output'];
  surroundFx1: Scalars['Int']['output'];
  surroundFx2: Scalars['Boolean']['output'];
  surroundMethod2: Scalars['Boolean']['output'];
  surroundMix: Scalars['Int']['output'];
  tagcacheAutoupdate: Scalars['Boolean']['output'];
  tagcacheDbPath: Scalars['String']['output'];
  tagcacheRam: Scalars['Int']['output'];
  tagcacheScanPaths: Scalars['String']['output'];
  talkBatteryLevel: Scalars['Boolean']['output'];
  talkDir: Scalars['Int']['output'];
  talkDirClip: Scalars['Boolean']['output'];
  talkFile: Scalars['Int']['output'];
  talkFileClip: Scalars['Boolean']['output'];
  talkFiletype: Scalars['Boolean']['output'];
  talkMenu: Scalars['Boolean']['output'];
  talkMixerAmp: Scalars['Int']['output'];
  timeformat: Scalars['Int']['output'];
  timestretchEnabled: Scalars['Boolean']['output'];
  touchMode: Scalars['Int']['output'];
  touchpadDeadzone: Scalars['Int']['output'];
  touchpadSensitivity: Scalars['Int']['output'];
  treble: Scalars['Int']['output'];
  trebleCutoff: Scalars['Int']['output'];
  unplugAutoresume: Scalars['Boolean']['output'];
  unplugMode: Scalars['Int']['output'];
  usbCharging: Scalars['Int']['output'];
  usbHid: Scalars['Boolean']['output'];
  usbKeypadMode: Scalars['Int']['output'];
  usbSkipFirstDrive: Scalars['Boolean']['output'];
  usemrb: Scalars['Int']['output'];
  viewersIconFile: Scalars['String']['output'];
  volume: Scalars['Int']['output'];
  volumeAdjustMode: Scalars['Int']['output'];
  volumeAdjustNormSteps: Scalars['Int']['output'];
  volumeLimit: Scalars['Int']['output'];
  volumeType: Scalars['Int']['output'];
  warnonEraseDynplaylist: Scalars['Boolean']['output'];
  wpsFile: Scalars['String']['output'];
};

export type GetEntriesQueryVariables = Exact<{
  path?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetEntriesQuery = { __typename?: 'Query', treeGetEntries: Array<{ __typename?: 'Entry', name: string, attr: number, timeWrite: number }> };

export type LikeTrackMutationVariables = Exact<{
  trackId: Scalars['String']['input'];
}>;


export type LikeTrackMutation = { __typename?: 'Mutation', likeTrack: number };

export type UnlikeTrackMutationVariables = Exact<{
  trackId: Scalars['String']['input'];
}>;


export type UnlikeTrackMutation = { __typename?: 'Mutation', unlikeTrack: number };

export type LikeAlbumMutationVariables = Exact<{
  albumId: Scalars['String']['input'];
}>;


export type LikeAlbumMutation = { __typename?: 'Mutation', likeAlbum: number };

export type UnlikeAlbumMutationVariables = Exact<{
  albumId: Scalars['String']['input'];
}>;


export type UnlikeAlbumMutation = { __typename?: 'Mutation', unlikeAlbum: number };

export type GetAlbumsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAlbumsQuery = { __typename?: 'Query', albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> }> };

export type GetArtistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetArtistsQuery = { __typename?: 'Query', artists: Array<{ __typename?: 'Artist', id: string, name: string }> };

export type GetArtistQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetArtistQuery = { __typename?: 'Query', artist?: { __typename?: 'Artist', id: string, name: string, albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string }>, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> } | null };

export type TracksQueryVariables = Exact<{ [key: string]: never; }>;


export type TracksQuery = { __typename?: 'Query', tracks: Array<{ __typename?: 'Track', id?: string | null, tracknum: number, title: string, artist: string, album: string, discnum: number, albumArtist: string, artistId?: string | null, albumId?: string | null, albumArt?: string | null, path: string, length: number }> };

export type GetAlbumQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetAlbumQuery = { __typename?: 'Query', album?: { __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, tracknum: number, artist: string, album: string, discnum: number, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> } | null };

export type GetLikedTracksQueryVariables = Exact<{ [key: string]: never; }>;


export type GetLikedTracksQuery = { __typename?: 'Query', likedTracks: Array<{ __typename?: 'Track', id?: string | null, tracknum: number, title: string, artist: string, album: string, discnum: number, albumArtist: string, artistId?: string | null, albumId?: string | null, albumArt?: string | null, path: string, length: number }> };

export type GetLikedAlbumsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetLikedAlbumsQuery = { __typename?: 'Query', likedAlbums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> }> };

export type PlayMutationVariables = Exact<{
  elapsed: Scalars['Int']['input'];
  offset: Scalars['Int']['input'];
}>;


export type PlayMutation = { __typename?: 'Mutation', play: number };

export type PauseMutationVariables = Exact<{ [key: string]: never; }>;


export type PauseMutation = { __typename?: 'Mutation', pause: number };

export type ResumeMutationVariables = Exact<{ [key: string]: never; }>;


export type ResumeMutation = { __typename?: 'Mutation', resume: number };

export type PreviousMutationVariables = Exact<{ [key: string]: never; }>;


export type PreviousMutation = { __typename?: 'Mutation', previous: number };

export type NextMutationVariables = Exact<{ [key: string]: never; }>;


export type NextMutation = { __typename?: 'Mutation', next: number };

export type PlayAlbumMutationVariables = Exact<{
  albumId: Scalars['String']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
}>;


export type PlayAlbumMutation = { __typename?: 'Mutation', playAlbum: number };

export type PlayArtistTracksMutationVariables = Exact<{
  artistId: Scalars['String']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
}>;


export type PlayArtistTracksMutation = { __typename?: 'Mutation', playArtistTracks: number };

export type PlayDirectoryMutationVariables = Exact<{
  path: Scalars['String']['input'];
  recurse?: InputMaybe<Scalars['Boolean']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
}>;


export type PlayDirectoryMutation = { __typename?: 'Mutation', playDirectory: number };

export type PlayTrackMutationVariables = Exact<{
  path: Scalars['String']['input'];
}>;


export type PlayTrackMutation = { __typename?: 'Mutation', playTrack: number };

export type GetCurrentTrackQueryVariables = Exact<{ [key: string]: never; }>;


export type GetCurrentTrackQuery = { __typename?: 'Query', currentTrack?: { __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, elapsed: number, length: number, year: number, yearString: string } | null };

export type GetNextTrackQueryVariables = Exact<{ [key: string]: never; }>;


export type GetNextTrackQuery = { __typename?: 'Query', nextTrack?: { __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, length: number, year: number, yearString: string } | null };

export type GetPlaybackStatusQueryVariables = Exact<{ [key: string]: never; }>;


export type GetPlaybackStatusQuery = { __typename?: 'Query', status: number };

export type CurrentlyPlayingSongSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type CurrentlyPlayingSongSubscription = { __typename?: 'Subscription', currentlyPlayingSong: { __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, elapsed: number, length: number, year: number, yearString: string } };

export type PlaybackStatusSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type PlaybackStatusSubscription = { __typename?: 'Subscription', playbackStatus: { __typename?: 'AudioStatus', status: number } };

export type ResumePlaylistMutationVariables = Exact<{ [key: string]: never; }>;


export type ResumePlaylistMutation = { __typename?: 'Mutation', playlistResume: number };

export type ResumePlaylistTrackMutationVariables = Exact<{ [key: string]: never; }>;


export type ResumePlaylistTrackMutation = { __typename?: 'Mutation', resumeTrack: string };

export type PlaylistRemoveTrackMutationVariables = Exact<{
  index: Scalars['Int']['input'];
}>;


export type PlaylistRemoveTrackMutation = { __typename?: 'Mutation', playlistRemoveTrack: number };

export type StartPlaylistMutationVariables = Exact<{
  startIndex?: InputMaybe<Scalars['Int']['input']>;
  elapsed?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
}>;


export type StartPlaylistMutation = { __typename?: 'Mutation', playlistStart: number };

export type InsertTracksMutationVariables = Exact<{
  playlistId?: InputMaybe<Scalars['String']['input']>;
  position: Scalars['Int']['input'];
  tracks: Array<Scalars['String']['input']> | Scalars['String']['input'];
}>;


export type InsertTracksMutation = { __typename?: 'Mutation', insertTracks: number };

export type InsertDirectoryMutationVariables = Exact<{
  playlistId?: InputMaybe<Scalars['String']['input']>;
  position: Scalars['Int']['input'];
  directory: Scalars['String']['input'];
}>;


export type InsertDirectoryMutation = { __typename?: 'Mutation', insertDirectory: number };

export type InsertAlbumMutationVariables = Exact<{
  albumId: Scalars['String']['input'];
  position: Scalars['Int']['input'];
}>;


export type InsertAlbumMutation = { __typename?: 'Mutation', insertAlbum: number };

export type GetCurrentPlaylistQueryVariables = Exact<{ [key: string]: never; }>;


export type GetCurrentPlaylistQuery = { __typename?: 'Query', playlistGetCurrent: { __typename?: 'Playlist', index: number, amount: number, maxPlaylistSize: number, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, path: string, album: string, length: number }> } };

export type PlaylistChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type PlaylistChangedSubscription = { __typename?: 'Subscription', playlistChanged: { __typename?: 'Playlist', index: number, amount: number, maxPlaylistSize: number, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, path: string }> } };

export type GetGlobalSettingsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetGlobalSettingsQuery = { __typename?: 'Query', globalSettings: { __typename?: 'UserSettings', volume: number, eqEnabled: boolean, eqBandSettings: Array<{ __typename?: 'EqBandSetting', q: number, cutoff: number, gain: number }> } };

export type AdjustVolumeMutationVariables = Exact<{
  steps: Scalars['Int']['input'];
}>;


export type AdjustVolumeMutation = { __typename?: 'Mutation', adjustVolume: number };

export type GetRockboxVersionQueryVariables = Exact<{ [key: string]: never; }>;


export type GetRockboxVersionQuery = { __typename?: 'Query', rockboxVersion: string };

export type GetGlobalStatusQueryVariables = Exact<{ [key: string]: never; }>;


export type GetGlobalStatusQuery = { __typename?: 'Query', globalStatus: { __typename?: 'SystemStatus', resumeIndex: number, resumeCrc32: number, resumeOffset: number, resumeElapsed: number } };


export const GetEntriesDocument = gql`
    query GetEntries($path: String) {
  treeGetEntries(path: $path) {
    name
    attr
    timeWrite
  }
}
    `;

/**
 * __useGetEntriesQuery__
 *
 * To run a query within a React component, call `useGetEntriesQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetEntriesQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetEntriesQuery({
 *   variables: {
 *      path: // value for 'path'
 *   },
 * });
 */
export function useGetEntriesQuery(baseOptions?: Apollo.QueryHookOptions<GetEntriesQuery, GetEntriesQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetEntriesQuery, GetEntriesQueryVariables>(GetEntriesDocument, options);
      }
export function useGetEntriesLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetEntriesQuery, GetEntriesQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetEntriesQuery, GetEntriesQueryVariables>(GetEntriesDocument, options);
        }
export function useGetEntriesSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetEntriesQuery, GetEntriesQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetEntriesQuery, GetEntriesQueryVariables>(GetEntriesDocument, options);
        }
export type GetEntriesQueryHookResult = ReturnType<typeof useGetEntriesQuery>;
export type GetEntriesLazyQueryHookResult = ReturnType<typeof useGetEntriesLazyQuery>;
export type GetEntriesSuspenseQueryHookResult = ReturnType<typeof useGetEntriesSuspenseQuery>;
export type GetEntriesQueryResult = Apollo.QueryResult<GetEntriesQuery, GetEntriesQueryVariables>;
export const LikeTrackDocument = gql`
    mutation LikeTrack($trackId: String!) {
  likeTrack(id: $trackId)
}
    `;
export type LikeTrackMutationFn = Apollo.MutationFunction<LikeTrackMutation, LikeTrackMutationVariables>;

/**
 * __useLikeTrackMutation__
 *
 * To run a mutation, you first call `useLikeTrackMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useLikeTrackMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [likeTrackMutation, { data, loading, error }] = useLikeTrackMutation({
 *   variables: {
 *      trackId: // value for 'trackId'
 *   },
 * });
 */
export function useLikeTrackMutation(baseOptions?: Apollo.MutationHookOptions<LikeTrackMutation, LikeTrackMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<LikeTrackMutation, LikeTrackMutationVariables>(LikeTrackDocument, options);
      }
export type LikeTrackMutationHookResult = ReturnType<typeof useLikeTrackMutation>;
export type LikeTrackMutationResult = Apollo.MutationResult<LikeTrackMutation>;
export type LikeTrackMutationOptions = Apollo.BaseMutationOptions<LikeTrackMutation, LikeTrackMutationVariables>;
export const UnlikeTrackDocument = gql`
    mutation UnlikeTrack($trackId: String!) {
  unlikeTrack(id: $trackId)
}
    `;
export type UnlikeTrackMutationFn = Apollo.MutationFunction<UnlikeTrackMutation, UnlikeTrackMutationVariables>;

/**
 * __useUnlikeTrackMutation__
 *
 * To run a mutation, you first call `useUnlikeTrackMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useUnlikeTrackMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [unlikeTrackMutation, { data, loading, error }] = useUnlikeTrackMutation({
 *   variables: {
 *      trackId: // value for 'trackId'
 *   },
 * });
 */
export function useUnlikeTrackMutation(baseOptions?: Apollo.MutationHookOptions<UnlikeTrackMutation, UnlikeTrackMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<UnlikeTrackMutation, UnlikeTrackMutationVariables>(UnlikeTrackDocument, options);
      }
export type UnlikeTrackMutationHookResult = ReturnType<typeof useUnlikeTrackMutation>;
export type UnlikeTrackMutationResult = Apollo.MutationResult<UnlikeTrackMutation>;
export type UnlikeTrackMutationOptions = Apollo.BaseMutationOptions<UnlikeTrackMutation, UnlikeTrackMutationVariables>;
export const LikeAlbumDocument = gql`
    mutation LikeAlbum($albumId: String!) {
  likeAlbum(id: $albumId)
}
    `;
export type LikeAlbumMutationFn = Apollo.MutationFunction<LikeAlbumMutation, LikeAlbumMutationVariables>;

/**
 * __useLikeAlbumMutation__
 *
 * To run a mutation, you first call `useLikeAlbumMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useLikeAlbumMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [likeAlbumMutation, { data, loading, error }] = useLikeAlbumMutation({
 *   variables: {
 *      albumId: // value for 'albumId'
 *   },
 * });
 */
export function useLikeAlbumMutation(baseOptions?: Apollo.MutationHookOptions<LikeAlbumMutation, LikeAlbumMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<LikeAlbumMutation, LikeAlbumMutationVariables>(LikeAlbumDocument, options);
      }
export type LikeAlbumMutationHookResult = ReturnType<typeof useLikeAlbumMutation>;
export type LikeAlbumMutationResult = Apollo.MutationResult<LikeAlbumMutation>;
export type LikeAlbumMutationOptions = Apollo.BaseMutationOptions<LikeAlbumMutation, LikeAlbumMutationVariables>;
export const UnlikeAlbumDocument = gql`
    mutation UnlikeAlbum($albumId: String!) {
  unlikeAlbum(id: $albumId)
}
    `;
export type UnlikeAlbumMutationFn = Apollo.MutationFunction<UnlikeAlbumMutation, UnlikeAlbumMutationVariables>;

/**
 * __useUnlikeAlbumMutation__
 *
 * To run a mutation, you first call `useUnlikeAlbumMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useUnlikeAlbumMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [unlikeAlbumMutation, { data, loading, error }] = useUnlikeAlbumMutation({
 *   variables: {
 *      albumId: // value for 'albumId'
 *   },
 * });
 */
export function useUnlikeAlbumMutation(baseOptions?: Apollo.MutationHookOptions<UnlikeAlbumMutation, UnlikeAlbumMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<UnlikeAlbumMutation, UnlikeAlbumMutationVariables>(UnlikeAlbumDocument, options);
      }
export type UnlikeAlbumMutationHookResult = ReturnType<typeof useUnlikeAlbumMutation>;
export type UnlikeAlbumMutationResult = Apollo.MutationResult<UnlikeAlbumMutation>;
export type UnlikeAlbumMutationOptions = Apollo.BaseMutationOptions<UnlikeAlbumMutation, UnlikeAlbumMutationVariables>;
export const GetAlbumsDocument = gql`
    query GetAlbums {
  albums {
    id
    title
    artist
    albumArt
    year
    yearString
    artistId
    md5
    tracks {
      id
      title
      artist
      album
      albumArtist
      artistId
      albumId
      path
      length
    }
  }
}
    `;

/**
 * __useGetAlbumsQuery__
 *
 * To run a query within a React component, call `useGetAlbumsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetAlbumsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetAlbumsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetAlbumsQuery(baseOptions?: Apollo.QueryHookOptions<GetAlbumsQuery, GetAlbumsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, options);
      }
export function useGetAlbumsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetAlbumsQuery, GetAlbumsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, options);
        }
export function useGetAlbumsSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetAlbumsQuery, GetAlbumsQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, options);
        }
export type GetAlbumsQueryHookResult = ReturnType<typeof useGetAlbumsQuery>;
export type GetAlbumsLazyQueryHookResult = ReturnType<typeof useGetAlbumsLazyQuery>;
export type GetAlbumsSuspenseQueryHookResult = ReturnType<typeof useGetAlbumsSuspenseQuery>;
export type GetAlbumsQueryResult = Apollo.QueryResult<GetAlbumsQuery, GetAlbumsQueryVariables>;
export const GetArtistsDocument = gql`
    query GetArtists {
  artists {
    id
    name
  }
}
    `;

/**
 * __useGetArtistsQuery__
 *
 * To run a query within a React component, call `useGetArtistsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetArtistsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetArtistsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetArtistsQuery(baseOptions?: Apollo.QueryHookOptions<GetArtistsQuery, GetArtistsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, options);
      }
export function useGetArtistsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetArtistsQuery, GetArtistsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, options);
        }
export function useGetArtistsSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetArtistsQuery, GetArtistsQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, options);
        }
export type GetArtistsQueryHookResult = ReturnType<typeof useGetArtistsQuery>;
export type GetArtistsLazyQueryHookResult = ReturnType<typeof useGetArtistsLazyQuery>;
export type GetArtistsSuspenseQueryHookResult = ReturnType<typeof useGetArtistsSuspenseQuery>;
export type GetArtistsQueryResult = Apollo.QueryResult<GetArtistsQuery, GetArtistsQueryVariables>;
export const GetArtistDocument = gql`
    query GetArtist($id: String!) {
  artist(id: $id) {
    id
    name
    albums {
      id
      title
      artist
      albumArt
      year
      yearString
      artistId
      md5
    }
    tracks {
      id
      title
      artist
      album
      albumArt
      albumArtist
      artistId
      albumId
      path
      length
    }
  }
}
    `;

/**
 * __useGetArtistQuery__
 *
 * To run a query within a React component, call `useGetArtistQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetArtistQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetArtistQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetArtistQuery(baseOptions: Apollo.QueryHookOptions<GetArtistQuery, GetArtistQueryVariables> & ({ variables: GetArtistQueryVariables; skip?: boolean; } | { skip: boolean; }) ) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, options);
      }
export function useGetArtistLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetArtistQuery, GetArtistQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, options);
        }
export function useGetArtistSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetArtistQuery, GetArtistQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, options);
        }
export type GetArtistQueryHookResult = ReturnType<typeof useGetArtistQuery>;
export type GetArtistLazyQueryHookResult = ReturnType<typeof useGetArtistLazyQuery>;
export type GetArtistSuspenseQueryHookResult = ReturnType<typeof useGetArtistSuspenseQuery>;
export type GetArtistQueryResult = Apollo.QueryResult<GetArtistQuery, GetArtistQueryVariables>;
export const TracksDocument = gql`
    query Tracks {
  tracks {
    id
    tracknum
    title
    artist
    album
    discnum
    albumArtist
    artistId
    albumId
    albumArt
    path
    length
  }
}
    `;

/**
 * __useTracksQuery__
 *
 * To run a query within a React component, call `useTracksQuery` and pass it any options that fit your needs.
 * When your component renders, `useTracksQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useTracksQuery({
 *   variables: {
 *   },
 * });
 */
export function useTracksQuery(baseOptions?: Apollo.QueryHookOptions<TracksQuery, TracksQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<TracksQuery, TracksQueryVariables>(TracksDocument, options);
      }
export function useTracksLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<TracksQuery, TracksQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<TracksQuery, TracksQueryVariables>(TracksDocument, options);
        }
export function useTracksSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<TracksQuery, TracksQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<TracksQuery, TracksQueryVariables>(TracksDocument, options);
        }
export type TracksQueryHookResult = ReturnType<typeof useTracksQuery>;
export type TracksLazyQueryHookResult = ReturnType<typeof useTracksLazyQuery>;
export type TracksSuspenseQueryHookResult = ReturnType<typeof useTracksSuspenseQuery>;
export type TracksQueryResult = Apollo.QueryResult<TracksQuery, TracksQueryVariables>;
export const GetAlbumDocument = gql`
    query GetAlbum($id: String!) {
  album(id: $id) {
    id
    title
    artist
    albumArt
    year
    yearString
    artistId
    md5
    tracks {
      id
      title
      tracknum
      artist
      album
      discnum
      albumArtist
      artistId
      albumId
      path
      length
    }
  }
}
    `;

/**
 * __useGetAlbumQuery__
 *
 * To run a query within a React component, call `useGetAlbumQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetAlbumQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetAlbumQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetAlbumQuery(baseOptions: Apollo.QueryHookOptions<GetAlbumQuery, GetAlbumQueryVariables> & ({ variables: GetAlbumQueryVariables; skip?: boolean; } | { skip: boolean; }) ) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, options);
      }
export function useGetAlbumLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetAlbumQuery, GetAlbumQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, options);
        }
export function useGetAlbumSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetAlbumQuery, GetAlbumQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, options);
        }
export type GetAlbumQueryHookResult = ReturnType<typeof useGetAlbumQuery>;
export type GetAlbumLazyQueryHookResult = ReturnType<typeof useGetAlbumLazyQuery>;
export type GetAlbumSuspenseQueryHookResult = ReturnType<typeof useGetAlbumSuspenseQuery>;
export type GetAlbumQueryResult = Apollo.QueryResult<GetAlbumQuery, GetAlbumQueryVariables>;
export const GetLikedTracksDocument = gql`
    query GetLikedTracks {
  likedTracks {
    id
    tracknum
    title
    artist
    album
    discnum
    albumArtist
    artistId
    albumId
    albumArt
    path
    length
  }
}
    `;

/**
 * __useGetLikedTracksQuery__
 *
 * To run a query within a React component, call `useGetLikedTracksQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetLikedTracksQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetLikedTracksQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetLikedTracksQuery(baseOptions?: Apollo.QueryHookOptions<GetLikedTracksQuery, GetLikedTracksQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetLikedTracksQuery, GetLikedTracksQueryVariables>(GetLikedTracksDocument, options);
      }
export function useGetLikedTracksLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetLikedTracksQuery, GetLikedTracksQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetLikedTracksQuery, GetLikedTracksQueryVariables>(GetLikedTracksDocument, options);
        }
export function useGetLikedTracksSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetLikedTracksQuery, GetLikedTracksQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetLikedTracksQuery, GetLikedTracksQueryVariables>(GetLikedTracksDocument, options);
        }
export type GetLikedTracksQueryHookResult = ReturnType<typeof useGetLikedTracksQuery>;
export type GetLikedTracksLazyQueryHookResult = ReturnType<typeof useGetLikedTracksLazyQuery>;
export type GetLikedTracksSuspenseQueryHookResult = ReturnType<typeof useGetLikedTracksSuspenseQuery>;
export type GetLikedTracksQueryResult = Apollo.QueryResult<GetLikedTracksQuery, GetLikedTracksQueryVariables>;
export const GetLikedAlbumsDocument = gql`
    query GetLikedAlbums {
  likedAlbums {
    id
    title
    artist
    albumArt
    year
    yearString
    artistId
    md5
    tracks {
      id
      title
      artist
      album
      albumArtist
      artistId
      albumId
      path
      length
    }
  }
}
    `;

/**
 * __useGetLikedAlbumsQuery__
 *
 * To run a query within a React component, call `useGetLikedAlbumsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetLikedAlbumsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetLikedAlbumsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetLikedAlbumsQuery(baseOptions?: Apollo.QueryHookOptions<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>(GetLikedAlbumsDocument, options);
      }
export function useGetLikedAlbumsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>(GetLikedAlbumsDocument, options);
        }
export function useGetLikedAlbumsSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>(GetLikedAlbumsDocument, options);
        }
export type GetLikedAlbumsQueryHookResult = ReturnType<typeof useGetLikedAlbumsQuery>;
export type GetLikedAlbumsLazyQueryHookResult = ReturnType<typeof useGetLikedAlbumsLazyQuery>;
export type GetLikedAlbumsSuspenseQueryHookResult = ReturnType<typeof useGetLikedAlbumsSuspenseQuery>;
export type GetLikedAlbumsQueryResult = Apollo.QueryResult<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>;
export const PlayDocument = gql`
    mutation Play($elapsed: Int!, $offset: Int!) {
  play(elapsed: $elapsed, offset: $offset)
}
    `;
export type PlayMutationFn = Apollo.MutationFunction<PlayMutation, PlayMutationVariables>;

/**
 * __usePlayMutation__
 *
 * To run a mutation, you first call `usePlayMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playMutation, { data, loading, error }] = usePlayMutation({
 *   variables: {
 *      elapsed: // value for 'elapsed'
 *      offset: // value for 'offset'
 *   },
 * });
 */
export function usePlayMutation(baseOptions?: Apollo.MutationHookOptions<PlayMutation, PlayMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayMutation, PlayMutationVariables>(PlayDocument, options);
      }
export type PlayMutationHookResult = ReturnType<typeof usePlayMutation>;
export type PlayMutationResult = Apollo.MutationResult<PlayMutation>;
export type PlayMutationOptions = Apollo.BaseMutationOptions<PlayMutation, PlayMutationVariables>;
export const PauseDocument = gql`
    mutation Pause {
  pause
}
    `;
export type PauseMutationFn = Apollo.MutationFunction<PauseMutation, PauseMutationVariables>;

/**
 * __usePauseMutation__
 *
 * To run a mutation, you first call `usePauseMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePauseMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [pauseMutation, { data, loading, error }] = usePauseMutation({
 *   variables: {
 *   },
 * });
 */
export function usePauseMutation(baseOptions?: Apollo.MutationHookOptions<PauseMutation, PauseMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PauseMutation, PauseMutationVariables>(PauseDocument, options);
      }
export type PauseMutationHookResult = ReturnType<typeof usePauseMutation>;
export type PauseMutationResult = Apollo.MutationResult<PauseMutation>;
export type PauseMutationOptions = Apollo.BaseMutationOptions<PauseMutation, PauseMutationVariables>;
export const ResumeDocument = gql`
    mutation Resume {
  resume
}
    `;
export type ResumeMutationFn = Apollo.MutationFunction<ResumeMutation, ResumeMutationVariables>;

/**
 * __useResumeMutation__
 *
 * To run a mutation, you first call `useResumeMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useResumeMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [resumeMutation, { data, loading, error }] = useResumeMutation({
 *   variables: {
 *   },
 * });
 */
export function useResumeMutation(baseOptions?: Apollo.MutationHookOptions<ResumeMutation, ResumeMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<ResumeMutation, ResumeMutationVariables>(ResumeDocument, options);
      }
export type ResumeMutationHookResult = ReturnType<typeof useResumeMutation>;
export type ResumeMutationResult = Apollo.MutationResult<ResumeMutation>;
export type ResumeMutationOptions = Apollo.BaseMutationOptions<ResumeMutation, ResumeMutationVariables>;
export const PreviousDocument = gql`
    mutation Previous {
  previous
}
    `;
export type PreviousMutationFn = Apollo.MutationFunction<PreviousMutation, PreviousMutationVariables>;

/**
 * __usePreviousMutation__
 *
 * To run a mutation, you first call `usePreviousMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePreviousMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [previousMutation, { data, loading, error }] = usePreviousMutation({
 *   variables: {
 *   },
 * });
 */
export function usePreviousMutation(baseOptions?: Apollo.MutationHookOptions<PreviousMutation, PreviousMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PreviousMutation, PreviousMutationVariables>(PreviousDocument, options);
      }
export type PreviousMutationHookResult = ReturnType<typeof usePreviousMutation>;
export type PreviousMutationResult = Apollo.MutationResult<PreviousMutation>;
export type PreviousMutationOptions = Apollo.BaseMutationOptions<PreviousMutation, PreviousMutationVariables>;
export const NextDocument = gql`
    mutation Next {
  next
}
    `;
export type NextMutationFn = Apollo.MutationFunction<NextMutation, NextMutationVariables>;

/**
 * __useNextMutation__
 *
 * To run a mutation, you first call `useNextMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useNextMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [nextMutation, { data, loading, error }] = useNextMutation({
 *   variables: {
 *   },
 * });
 */
export function useNextMutation(baseOptions?: Apollo.MutationHookOptions<NextMutation, NextMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<NextMutation, NextMutationVariables>(NextDocument, options);
      }
export type NextMutationHookResult = ReturnType<typeof useNextMutation>;
export type NextMutationResult = Apollo.MutationResult<NextMutation>;
export type NextMutationOptions = Apollo.BaseMutationOptions<NextMutation, NextMutationVariables>;
export const PlayAlbumDocument = gql`
    mutation PlayAlbum($albumId: String!, $shuffle: Boolean) {
  playAlbum(albumId: $albumId, shuffle: $shuffle)
}
    `;
export type PlayAlbumMutationFn = Apollo.MutationFunction<PlayAlbumMutation, PlayAlbumMutationVariables>;

/**
 * __usePlayAlbumMutation__
 *
 * To run a mutation, you first call `usePlayAlbumMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayAlbumMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playAlbumMutation, { data, loading, error }] = usePlayAlbumMutation({
 *   variables: {
 *      albumId: // value for 'albumId'
 *      shuffle: // value for 'shuffle'
 *   },
 * });
 */
export function usePlayAlbumMutation(baseOptions?: Apollo.MutationHookOptions<PlayAlbumMutation, PlayAlbumMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayAlbumMutation, PlayAlbumMutationVariables>(PlayAlbumDocument, options);
      }
export type PlayAlbumMutationHookResult = ReturnType<typeof usePlayAlbumMutation>;
export type PlayAlbumMutationResult = Apollo.MutationResult<PlayAlbumMutation>;
export type PlayAlbumMutationOptions = Apollo.BaseMutationOptions<PlayAlbumMutation, PlayAlbumMutationVariables>;
export const PlayArtistTracksDocument = gql`
    mutation PlayArtistTracks($artistId: String!, $shuffle: Boolean) {
  playArtistTracks(artistId: $artistId, shuffle: $shuffle)
}
    `;
export type PlayArtistTracksMutationFn = Apollo.MutationFunction<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>;

/**
 * __usePlayArtistTracksMutation__
 *
 * To run a mutation, you first call `usePlayArtistTracksMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayArtistTracksMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playArtistTracksMutation, { data, loading, error }] = usePlayArtistTracksMutation({
 *   variables: {
 *      artistId: // value for 'artistId'
 *      shuffle: // value for 'shuffle'
 *   },
 * });
 */
export function usePlayArtistTracksMutation(baseOptions?: Apollo.MutationHookOptions<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>(PlayArtistTracksDocument, options);
      }
export type PlayArtistTracksMutationHookResult = ReturnType<typeof usePlayArtistTracksMutation>;
export type PlayArtistTracksMutationResult = Apollo.MutationResult<PlayArtistTracksMutation>;
export type PlayArtistTracksMutationOptions = Apollo.BaseMutationOptions<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>;
export const PlayDirectoryDocument = gql`
    mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean) {
  playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle)
}
    `;
export type PlayDirectoryMutationFn = Apollo.MutationFunction<PlayDirectoryMutation, PlayDirectoryMutationVariables>;

/**
 * __usePlayDirectoryMutation__
 *
 * To run a mutation, you first call `usePlayDirectoryMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayDirectoryMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playDirectoryMutation, { data, loading, error }] = usePlayDirectoryMutation({
 *   variables: {
 *      path: // value for 'path'
 *      recurse: // value for 'recurse'
 *      shuffle: // value for 'shuffle'
 *   },
 * });
 */
export function usePlayDirectoryMutation(baseOptions?: Apollo.MutationHookOptions<PlayDirectoryMutation, PlayDirectoryMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayDirectoryMutation, PlayDirectoryMutationVariables>(PlayDirectoryDocument, options);
      }
export type PlayDirectoryMutationHookResult = ReturnType<typeof usePlayDirectoryMutation>;
export type PlayDirectoryMutationResult = Apollo.MutationResult<PlayDirectoryMutation>;
export type PlayDirectoryMutationOptions = Apollo.BaseMutationOptions<PlayDirectoryMutation, PlayDirectoryMutationVariables>;
export const PlayTrackDocument = gql`
    mutation PlayTrack($path: String!) {
  playTrack(path: $path)
}
    `;
export type PlayTrackMutationFn = Apollo.MutationFunction<PlayTrackMutation, PlayTrackMutationVariables>;

/**
 * __usePlayTrackMutation__
 *
 * To run a mutation, you first call `usePlayTrackMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayTrackMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playTrackMutation, { data, loading, error }] = usePlayTrackMutation({
 *   variables: {
 *      path: // value for 'path'
 *   },
 * });
 */
export function usePlayTrackMutation(baseOptions?: Apollo.MutationHookOptions<PlayTrackMutation, PlayTrackMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayTrackMutation, PlayTrackMutationVariables>(PlayTrackDocument, options);
      }
export type PlayTrackMutationHookResult = ReturnType<typeof usePlayTrackMutation>;
export type PlayTrackMutationResult = Apollo.MutationResult<PlayTrackMutation>;
export type PlayTrackMutationOptions = Apollo.BaseMutationOptions<PlayTrackMutation, PlayTrackMutationVariables>;
export const GetCurrentTrackDocument = gql`
    query GetCurrentTrack {
  currentTrack {
    id
    title
    artist
    album
    albumArt
    artistId
    albumId
    elapsed
    length
    year
    yearString
  }
}
    `;

/**
 * __useGetCurrentTrackQuery__
 *
 * To run a query within a React component, call `useGetCurrentTrackQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetCurrentTrackQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetCurrentTrackQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetCurrentTrackQuery(baseOptions?: Apollo.QueryHookOptions<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>(GetCurrentTrackDocument, options);
      }
export function useGetCurrentTrackLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>(GetCurrentTrackDocument, options);
        }
export function useGetCurrentTrackSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>(GetCurrentTrackDocument, options);
        }
export type GetCurrentTrackQueryHookResult = ReturnType<typeof useGetCurrentTrackQuery>;
export type GetCurrentTrackLazyQueryHookResult = ReturnType<typeof useGetCurrentTrackLazyQuery>;
export type GetCurrentTrackSuspenseQueryHookResult = ReturnType<typeof useGetCurrentTrackSuspenseQuery>;
export type GetCurrentTrackQueryResult = Apollo.QueryResult<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>;
export const GetNextTrackDocument = gql`
    query GetNextTrack {
  nextTrack {
    id
    title
    artist
    album
    albumArt
    artistId
    albumId
    length
    year
    yearString
  }
}
    `;

/**
 * __useGetNextTrackQuery__
 *
 * To run a query within a React component, call `useGetNextTrackQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetNextTrackQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetNextTrackQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetNextTrackQuery(baseOptions?: Apollo.QueryHookOptions<GetNextTrackQuery, GetNextTrackQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetNextTrackQuery, GetNextTrackQueryVariables>(GetNextTrackDocument, options);
      }
export function useGetNextTrackLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetNextTrackQuery, GetNextTrackQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetNextTrackQuery, GetNextTrackQueryVariables>(GetNextTrackDocument, options);
        }
export function useGetNextTrackSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetNextTrackQuery, GetNextTrackQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetNextTrackQuery, GetNextTrackQueryVariables>(GetNextTrackDocument, options);
        }
export type GetNextTrackQueryHookResult = ReturnType<typeof useGetNextTrackQuery>;
export type GetNextTrackLazyQueryHookResult = ReturnType<typeof useGetNextTrackLazyQuery>;
export type GetNextTrackSuspenseQueryHookResult = ReturnType<typeof useGetNextTrackSuspenseQuery>;
export type GetNextTrackQueryResult = Apollo.QueryResult<GetNextTrackQuery, GetNextTrackQueryVariables>;
export const GetPlaybackStatusDocument = gql`
    query GetPlaybackStatus {
  status
}
    `;

/**
 * __useGetPlaybackStatusQuery__
 *
 * To run a query within a React component, call `useGetPlaybackStatusQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetPlaybackStatusQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetPlaybackStatusQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetPlaybackStatusQuery(baseOptions?: Apollo.QueryHookOptions<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>(GetPlaybackStatusDocument, options);
      }
export function useGetPlaybackStatusLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>(GetPlaybackStatusDocument, options);
        }
export function useGetPlaybackStatusSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>(GetPlaybackStatusDocument, options);
        }
export type GetPlaybackStatusQueryHookResult = ReturnType<typeof useGetPlaybackStatusQuery>;
export type GetPlaybackStatusLazyQueryHookResult = ReturnType<typeof useGetPlaybackStatusLazyQuery>;
export type GetPlaybackStatusSuspenseQueryHookResult = ReturnType<typeof useGetPlaybackStatusSuspenseQuery>;
export type GetPlaybackStatusQueryResult = Apollo.QueryResult<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>;
export const CurrentlyPlayingSongDocument = gql`
    subscription CurrentlyPlayingSong {
  currentlyPlayingSong {
    id
    title
    artist
    album
    albumArt
    artistId
    albumId
    elapsed
    length
    year
    yearString
  }
}
    `;

/**
 * __useCurrentlyPlayingSongSubscription__
 *
 * To run a query within a React component, call `useCurrentlyPlayingSongSubscription` and pass it any options that fit your needs.
 * When your component renders, `useCurrentlyPlayingSongSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useCurrentlyPlayingSongSubscription({
 *   variables: {
 *   },
 * });
 */
export function useCurrentlyPlayingSongSubscription(baseOptions?: Apollo.SubscriptionHookOptions<CurrentlyPlayingSongSubscription, CurrentlyPlayingSongSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<CurrentlyPlayingSongSubscription, CurrentlyPlayingSongSubscriptionVariables>(CurrentlyPlayingSongDocument, options);
      }
export type CurrentlyPlayingSongSubscriptionHookResult = ReturnType<typeof useCurrentlyPlayingSongSubscription>;
export type CurrentlyPlayingSongSubscriptionResult = Apollo.SubscriptionResult<CurrentlyPlayingSongSubscription>;
export const PlaybackStatusDocument = gql`
    subscription PlaybackStatus {
  playbackStatus {
    status
  }
}
    `;

/**
 * __usePlaybackStatusSubscription__
 *
 * To run a query within a React component, call `usePlaybackStatusSubscription` and pass it any options that fit your needs.
 * When your component renders, `usePlaybackStatusSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = usePlaybackStatusSubscription({
 *   variables: {
 *   },
 * });
 */
export function usePlaybackStatusSubscription(baseOptions?: Apollo.SubscriptionHookOptions<PlaybackStatusSubscription, PlaybackStatusSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<PlaybackStatusSubscription, PlaybackStatusSubscriptionVariables>(PlaybackStatusDocument, options);
      }
export type PlaybackStatusSubscriptionHookResult = ReturnType<typeof usePlaybackStatusSubscription>;
export type PlaybackStatusSubscriptionResult = Apollo.SubscriptionResult<PlaybackStatusSubscription>;
export const ResumePlaylistDocument = gql`
    mutation ResumePlaylist {
  playlistResume
}
    `;
export type ResumePlaylistMutationFn = Apollo.MutationFunction<ResumePlaylistMutation, ResumePlaylistMutationVariables>;

/**
 * __useResumePlaylistMutation__
 *
 * To run a mutation, you first call `useResumePlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useResumePlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [resumePlaylistMutation, { data, loading, error }] = useResumePlaylistMutation({
 *   variables: {
 *   },
 * });
 */
export function useResumePlaylistMutation(baseOptions?: Apollo.MutationHookOptions<ResumePlaylistMutation, ResumePlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<ResumePlaylistMutation, ResumePlaylistMutationVariables>(ResumePlaylistDocument, options);
      }
export type ResumePlaylistMutationHookResult = ReturnType<typeof useResumePlaylistMutation>;
export type ResumePlaylistMutationResult = Apollo.MutationResult<ResumePlaylistMutation>;
export type ResumePlaylistMutationOptions = Apollo.BaseMutationOptions<ResumePlaylistMutation, ResumePlaylistMutationVariables>;
export const ResumePlaylistTrackDocument = gql`
    mutation ResumePlaylistTrack {
  resumeTrack
}
    `;
export type ResumePlaylistTrackMutationFn = Apollo.MutationFunction<ResumePlaylistTrackMutation, ResumePlaylistTrackMutationVariables>;

/**
 * __useResumePlaylistTrackMutation__
 *
 * To run a mutation, you first call `useResumePlaylistTrackMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useResumePlaylistTrackMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [resumePlaylistTrackMutation, { data, loading, error }] = useResumePlaylistTrackMutation({
 *   variables: {
 *   },
 * });
 */
export function useResumePlaylistTrackMutation(baseOptions?: Apollo.MutationHookOptions<ResumePlaylistTrackMutation, ResumePlaylistTrackMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<ResumePlaylistTrackMutation, ResumePlaylistTrackMutationVariables>(ResumePlaylistTrackDocument, options);
      }
export type ResumePlaylistTrackMutationHookResult = ReturnType<typeof useResumePlaylistTrackMutation>;
export type ResumePlaylistTrackMutationResult = Apollo.MutationResult<ResumePlaylistTrackMutation>;
export type ResumePlaylistTrackMutationOptions = Apollo.BaseMutationOptions<ResumePlaylistTrackMutation, ResumePlaylistTrackMutationVariables>;
export const PlaylistRemoveTrackDocument = gql`
    mutation PlaylistRemoveTrack($index: Int!) {
  playlistRemoveTrack(index: $index)
}
    `;
export type PlaylistRemoveTrackMutationFn = Apollo.MutationFunction<PlaylistRemoveTrackMutation, PlaylistRemoveTrackMutationVariables>;

/**
 * __usePlaylistRemoveTrackMutation__
 *
 * To run a mutation, you first call `usePlaylistRemoveTrackMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlaylistRemoveTrackMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playlistRemoveTrackMutation, { data, loading, error }] = usePlaylistRemoveTrackMutation({
 *   variables: {
 *      index: // value for 'index'
 *   },
 * });
 */
export function usePlaylistRemoveTrackMutation(baseOptions?: Apollo.MutationHookOptions<PlaylistRemoveTrackMutation, PlaylistRemoveTrackMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlaylistRemoveTrackMutation, PlaylistRemoveTrackMutationVariables>(PlaylistRemoveTrackDocument, options);
      }
export type PlaylistRemoveTrackMutationHookResult = ReturnType<typeof usePlaylistRemoveTrackMutation>;
export type PlaylistRemoveTrackMutationResult = Apollo.MutationResult<PlaylistRemoveTrackMutation>;
export type PlaylistRemoveTrackMutationOptions = Apollo.BaseMutationOptions<PlaylistRemoveTrackMutation, PlaylistRemoveTrackMutationVariables>;
export const StartPlaylistDocument = gql`
    mutation StartPlaylist($startIndex: Int, $elapsed: Int, $offset: Int) {
  playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset)
}
    `;
export type StartPlaylistMutationFn = Apollo.MutationFunction<StartPlaylistMutation, StartPlaylistMutationVariables>;

/**
 * __useStartPlaylistMutation__
 *
 * To run a mutation, you first call `useStartPlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useStartPlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [startPlaylistMutation, { data, loading, error }] = useStartPlaylistMutation({
 *   variables: {
 *      startIndex: // value for 'startIndex'
 *      elapsed: // value for 'elapsed'
 *      offset: // value for 'offset'
 *   },
 * });
 */
export function useStartPlaylistMutation(baseOptions?: Apollo.MutationHookOptions<StartPlaylistMutation, StartPlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<StartPlaylistMutation, StartPlaylistMutationVariables>(StartPlaylistDocument, options);
      }
export type StartPlaylistMutationHookResult = ReturnType<typeof useStartPlaylistMutation>;
export type StartPlaylistMutationResult = Apollo.MutationResult<StartPlaylistMutation>;
export type StartPlaylistMutationOptions = Apollo.BaseMutationOptions<StartPlaylistMutation, StartPlaylistMutationVariables>;
export const InsertTracksDocument = gql`
    mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) {
  insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks)
}
    `;
export type InsertTracksMutationFn = Apollo.MutationFunction<InsertTracksMutation, InsertTracksMutationVariables>;

/**
 * __useInsertTracksMutation__
 *
 * To run a mutation, you first call `useInsertTracksMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useInsertTracksMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [insertTracksMutation, { data, loading, error }] = useInsertTracksMutation({
 *   variables: {
 *      playlistId: // value for 'playlistId'
 *      position: // value for 'position'
 *      tracks: // value for 'tracks'
 *   },
 * });
 */
export function useInsertTracksMutation(baseOptions?: Apollo.MutationHookOptions<InsertTracksMutation, InsertTracksMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<InsertTracksMutation, InsertTracksMutationVariables>(InsertTracksDocument, options);
      }
export type InsertTracksMutationHookResult = ReturnType<typeof useInsertTracksMutation>;
export type InsertTracksMutationResult = Apollo.MutationResult<InsertTracksMutation>;
export type InsertTracksMutationOptions = Apollo.BaseMutationOptions<InsertTracksMutation, InsertTracksMutationVariables>;
export const InsertDirectoryDocument = gql`
    mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) {
  insertDirectory(
    playlistId: $playlistId
    position: $position
    directory: $directory
  )
}
    `;
export type InsertDirectoryMutationFn = Apollo.MutationFunction<InsertDirectoryMutation, InsertDirectoryMutationVariables>;

/**
 * __useInsertDirectoryMutation__
 *
 * To run a mutation, you first call `useInsertDirectoryMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useInsertDirectoryMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [insertDirectoryMutation, { data, loading, error }] = useInsertDirectoryMutation({
 *   variables: {
 *      playlistId: // value for 'playlistId'
 *      position: // value for 'position'
 *      directory: // value for 'directory'
 *   },
 * });
 */
export function useInsertDirectoryMutation(baseOptions?: Apollo.MutationHookOptions<InsertDirectoryMutation, InsertDirectoryMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<InsertDirectoryMutation, InsertDirectoryMutationVariables>(InsertDirectoryDocument, options);
      }
export type InsertDirectoryMutationHookResult = ReturnType<typeof useInsertDirectoryMutation>;
export type InsertDirectoryMutationResult = Apollo.MutationResult<InsertDirectoryMutation>;
export type InsertDirectoryMutationOptions = Apollo.BaseMutationOptions<InsertDirectoryMutation, InsertDirectoryMutationVariables>;
export const InsertAlbumDocument = gql`
    mutation InsertAlbum($albumId: String!, $position: Int!) {
  insertAlbum(albumId: $albumId, position: $position)
}
    `;
export type InsertAlbumMutationFn = Apollo.MutationFunction<InsertAlbumMutation, InsertAlbumMutationVariables>;

/**
 * __useInsertAlbumMutation__
 *
 * To run a mutation, you first call `useInsertAlbumMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useInsertAlbumMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [insertAlbumMutation, { data, loading, error }] = useInsertAlbumMutation({
 *   variables: {
 *      albumId: // value for 'albumId'
 *      position: // value for 'position'
 *   },
 * });
 */
export function useInsertAlbumMutation(baseOptions?: Apollo.MutationHookOptions<InsertAlbumMutation, InsertAlbumMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<InsertAlbumMutation, InsertAlbumMutationVariables>(InsertAlbumDocument, options);
      }
export type InsertAlbumMutationHookResult = ReturnType<typeof useInsertAlbumMutation>;
export type InsertAlbumMutationResult = Apollo.MutationResult<InsertAlbumMutation>;
export type InsertAlbumMutationOptions = Apollo.BaseMutationOptions<InsertAlbumMutation, InsertAlbumMutationVariables>;
export const GetCurrentPlaylistDocument = gql`
    query GetCurrentPlaylist {
  playlistGetCurrent {
    index
    amount
    maxPlaylistSize
    tracks {
      id
      title
      artist
      albumArt
      artistId
      albumId
      path
      album
      length
    }
  }
}
    `;

/**
 * __useGetCurrentPlaylistQuery__
 *
 * To run a query within a React component, call `useGetCurrentPlaylistQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetCurrentPlaylistQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetCurrentPlaylistQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetCurrentPlaylistQuery(baseOptions?: Apollo.QueryHookOptions<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>(GetCurrentPlaylistDocument, options);
      }
export function useGetCurrentPlaylistLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>(GetCurrentPlaylistDocument, options);
        }
export function useGetCurrentPlaylistSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>(GetCurrentPlaylistDocument, options);
        }
export type GetCurrentPlaylistQueryHookResult = ReturnType<typeof useGetCurrentPlaylistQuery>;
export type GetCurrentPlaylistLazyQueryHookResult = ReturnType<typeof useGetCurrentPlaylistLazyQuery>;
export type GetCurrentPlaylistSuspenseQueryHookResult = ReturnType<typeof useGetCurrentPlaylistSuspenseQuery>;
export type GetCurrentPlaylistQueryResult = Apollo.QueryResult<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>;
export const PlaylistChangedDocument = gql`
    subscription PlaylistChanged {
  playlistChanged {
    index
    amount
    maxPlaylistSize
    tracks {
      id
      title
      artist
      albumArt
      artistId
      albumId
      path
    }
  }
}
    `;

/**
 * __usePlaylistChangedSubscription__
 *
 * To run a query within a React component, call `usePlaylistChangedSubscription` and pass it any options that fit your needs.
 * When your component renders, `usePlaylistChangedSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = usePlaylistChangedSubscription({
 *   variables: {
 *   },
 * });
 */
export function usePlaylistChangedSubscription(baseOptions?: Apollo.SubscriptionHookOptions<PlaylistChangedSubscription, PlaylistChangedSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<PlaylistChangedSubscription, PlaylistChangedSubscriptionVariables>(PlaylistChangedDocument, options);
      }
export type PlaylistChangedSubscriptionHookResult = ReturnType<typeof usePlaylistChangedSubscription>;
export type PlaylistChangedSubscriptionResult = Apollo.SubscriptionResult<PlaylistChangedSubscription>;
export const GetGlobalSettingsDocument = gql`
    query GetGlobalSettings {
  globalSettings {
    volume
    eqEnabled
    eqBandSettings {
      q
      cutoff
      gain
    }
  }
}
    `;

/**
 * __useGetGlobalSettingsQuery__
 *
 * To run a query within a React component, call `useGetGlobalSettingsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetGlobalSettingsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetGlobalSettingsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetGlobalSettingsQuery(baseOptions?: Apollo.QueryHookOptions<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>(GetGlobalSettingsDocument, options);
      }
export function useGetGlobalSettingsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>(GetGlobalSettingsDocument, options);
        }
export function useGetGlobalSettingsSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>(GetGlobalSettingsDocument, options);
        }
export type GetGlobalSettingsQueryHookResult = ReturnType<typeof useGetGlobalSettingsQuery>;
export type GetGlobalSettingsLazyQueryHookResult = ReturnType<typeof useGetGlobalSettingsLazyQuery>;
export type GetGlobalSettingsSuspenseQueryHookResult = ReturnType<typeof useGetGlobalSettingsSuspenseQuery>;
export type GetGlobalSettingsQueryResult = Apollo.QueryResult<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>;
export const AdjustVolumeDocument = gql`
    mutation AdjustVolume($steps: Int!) {
  adjustVolume(steps: $steps)
}
    `;
export type AdjustVolumeMutationFn = Apollo.MutationFunction<AdjustVolumeMutation, AdjustVolumeMutationVariables>;

/**
 * __useAdjustVolumeMutation__
 *
 * To run a mutation, you first call `useAdjustVolumeMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useAdjustVolumeMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [adjustVolumeMutation, { data, loading, error }] = useAdjustVolumeMutation({
 *   variables: {
 *      steps: // value for 'steps'
 *   },
 * });
 */
export function useAdjustVolumeMutation(baseOptions?: Apollo.MutationHookOptions<AdjustVolumeMutation, AdjustVolumeMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<AdjustVolumeMutation, AdjustVolumeMutationVariables>(AdjustVolumeDocument, options);
      }
export type AdjustVolumeMutationHookResult = ReturnType<typeof useAdjustVolumeMutation>;
export type AdjustVolumeMutationResult = Apollo.MutationResult<AdjustVolumeMutation>;
export type AdjustVolumeMutationOptions = Apollo.BaseMutationOptions<AdjustVolumeMutation, AdjustVolumeMutationVariables>;
export const GetRockboxVersionDocument = gql`
    query GetRockboxVersion {
  rockboxVersion
}
    `;

/**
 * __useGetRockboxVersionQuery__
 *
 * To run a query within a React component, call `useGetRockboxVersionQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetRockboxVersionQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetRockboxVersionQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetRockboxVersionQuery(baseOptions?: Apollo.QueryHookOptions<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>(GetRockboxVersionDocument, options);
      }
export function useGetRockboxVersionLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>(GetRockboxVersionDocument, options);
        }
export function useGetRockboxVersionSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>(GetRockboxVersionDocument, options);
        }
export type GetRockboxVersionQueryHookResult = ReturnType<typeof useGetRockboxVersionQuery>;
export type GetRockboxVersionLazyQueryHookResult = ReturnType<typeof useGetRockboxVersionLazyQuery>;
export type GetRockboxVersionSuspenseQueryHookResult = ReturnType<typeof useGetRockboxVersionSuspenseQuery>;
export type GetRockboxVersionQueryResult = Apollo.QueryResult<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>;
export const GetGlobalStatusDocument = gql`
    query GetGlobalStatus {
  globalStatus {
    resumeIndex
    resumeCrc32
    resumeOffset
    resumeElapsed
  }
}
    `;

/**
 * __useGetGlobalStatusQuery__
 *
 * To run a query within a React component, call `useGetGlobalStatusQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetGlobalStatusQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetGlobalStatusQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetGlobalStatusQuery(baseOptions?: Apollo.QueryHookOptions<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>(GetGlobalStatusDocument, options);
      }
export function useGetGlobalStatusLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>(GetGlobalStatusDocument, options);
        }
export function useGetGlobalStatusSuspenseQuery(baseOptions?: Apollo.SkipToken | Apollo.SuspenseQueryHookOptions<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>) {
          const options = baseOptions === Apollo.skipToken ? baseOptions : {...defaultOptions, ...baseOptions}
          return Apollo.useSuspenseQuery<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>(GetGlobalStatusDocument, options);
        }
export type GetGlobalStatusQueryHookResult = ReturnType<typeof useGetGlobalStatusQuery>;
export type GetGlobalStatusLazyQueryHookResult = ReturnType<typeof useGetGlobalStatusLazyQuery>;
export type GetGlobalStatusSuspenseQueryHookResult = ReturnType<typeof useGetGlobalStatusSuspenseQuery>;
export type GetGlobalStatusQueryResult = Apollo.QueryResult<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>;