import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query';
import { fetchData, TypedDocumentString } from '../lib/graphql-client';
import { useSubscription } from '../lib/subscription-client';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
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

export type CompressorSettingsInput = {
  attackTime: Scalars['Int']['input'];
  knee: Scalars['Int']['input'];
  makeupGain: Scalars['Int']['input'];
  ratio: Scalars['Int']['input'];
  releaseTime: Scalars['Int']['input'];
  threshold: Scalars['Int']['input'];
};

export type Device = {
  __typename?: 'Device';
  app: Scalars['String']['output'];
  baseUrl?: Maybe<Scalars['String']['output']>;
  host: Scalars['String']['output'];
  id: Scalars['String']['output'];
  ip: Scalars['String']['output'];
  isCastDevice: Scalars['Boolean']['output'];
  isConnected: Scalars['Boolean']['output'];
  isCurrentDevice: Scalars['Boolean']['output'];
  isSourceDevice: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  port: Scalars['Int']['output'];
  service: Scalars['String']['output'];
};

export type Entry = {
  __typename?: 'Entry';
  attr: Scalars['Int']['output'];
  customaction: Scalars['Int']['output'];
  displayName?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  timeWrite: Scalars['Int']['output'];
};

export type EqBandSetting = {
  __typename?: 'EqBandSetting';
  cutoff: Scalars['Int']['output'];
  gain: Scalars['Int']['output'];
  q: Scalars['Int']['output'];
};

export type EqBandSettingInput = {
  cutoff: Scalars['Int']['input'];
  gain: Scalars['Int']['input'];
  q: Scalars['Int']['input'];
};

export type Mutation = {
  __typename?: 'Mutation';
  addTracksToSavedPlaylist: Scalars['Boolean']['output'];
  adjustVolume: Scalars['Int']['output'];
  beepPlay: Scalars['String']['output'];
  connect: Scalars['Boolean']['output'];
  createPlaylistFolder: SavedPlaylistFolder;
  createSavedPlaylist: SavedPlaylist;
  createSmartPlaylist: SmartPlaylist;
  deletePlaylistFolder: Scalars['Boolean']['output'];
  deleteSavedPlaylist: Scalars['Boolean']['output'];
  deleteSmartPlaylist: Scalars['Boolean']['output'];
  disconnect: Scalars['Boolean']['output'];
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
  playAllTracks: Scalars['Int']['output'];
  playArtistTracks: Scalars['Int']['output'];
  playDirectory: Scalars['Int']['output'];
  playLikedTracks: Scalars['Int']['output'];
  playPlaylist: Scalars['Int']['output'];
  playSavedPlaylist: Scalars['Boolean']['output'];
  playSmartPlaylist: Scalars['Boolean']['output'];
  playTrack: Scalars['Int']['output'];
  playlistCreate: Scalars['Int']['output'];
  playlistRemoveAllTracks: Scalars['Int']['output'];
  playlistRemoveTrack: Scalars['Int']['output'];
  playlistResume: Scalars['Int']['output'];
  playlistSetModified: Scalars['String']['output'];
  playlistStart: Scalars['Int']['output'];
  playlistSync: Scalars['String']['output'];
  previous: Scalars['Int']['output'];
  recordTrackPlayed: Scalars['Boolean']['output'];
  recordTrackSkipped: Scalars['Boolean']['output'];
  removeTrackFromSavedPlaylist: Scalars['Boolean']['output'];
  resume: Scalars['Int']['output'];
  resumeTrack: Scalars['String']['output'];
  saveSettings: Scalars['Boolean']['output'];
  scanLibrary: Scalars['Int']['output'];
  setPitch: Scalars['String']['output'];
  shufflePlaylist: Scalars['Int']['output'];
  soundMax: Scalars['String']['output'];
  soundMin: Scalars['String']['output'];
  soundSet: Scalars['String']['output'];
  soundUnit: Scalars['String']['output'];
  systemSoundPlay: Scalars['String']['output'];
  unlikeAlbum: Scalars['Int']['output'];
  unlikeTrack: Scalars['Int']['output'];
  updateSavedPlaylist: Scalars['Boolean']['output'];
  updateSmartPlaylist: Scalars['Boolean']['output'];
};


export type MutationAddTracksToSavedPlaylistArgs = {
  playlistId: Scalars['String']['input'];
  trackIds: Array<Scalars['String']['input']>;
};


export type MutationAdjustVolumeArgs = {
  steps: Scalars['Int']['input'];
};


export type MutationConnectArgs = {
  id: Scalars['String']['input'];
};


export type MutationCreatePlaylistFolderArgs = {
  name: Scalars['String']['input'];
};


export type MutationCreateSavedPlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  folderId?: InputMaybe<Scalars['String']['input']>;
  image?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
  trackIds?: InputMaybe<Array<Scalars['String']['input']>>;
};


export type MutationCreateSmartPlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  folderId?: InputMaybe<Scalars['String']['input']>;
  image?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
  rules: Scalars['String']['input'];
};


export type MutationDeletePlaylistFolderArgs = {
  id: Scalars['String']['input'];
};


export type MutationDeleteSavedPlaylistArgs = {
  id: Scalars['String']['input'];
};


export type MutationDeleteSmartPlaylistArgs = {
  id: Scalars['String']['input'];
};


export type MutationDisconnectArgs = {
  id: Scalars['String']['input'];
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
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayAllTracksArgs = {
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayArtistTracksArgs = {
  artistId: Scalars['String']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayDirectoryArgs = {
  path: Scalars['String']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  recurse?: InputMaybe<Scalars['Boolean']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayLikedTracksArgs = {
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlayPlaylistArgs = {
  playlistId: Scalars['String']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationPlaySavedPlaylistArgs = {
  playlistId: Scalars['String']['input'];
};


export type MutationPlaySmartPlaylistArgs = {
  id: Scalars['String']['input'];
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


export type MutationRecordTrackPlayedArgs = {
  trackId: Scalars['String']['input'];
};


export type MutationRecordTrackSkippedArgs = {
  trackId: Scalars['String']['input'];
};


export type MutationRemoveTrackFromSavedPlaylistArgs = {
  playlistId: Scalars['String']['input'];
  trackId: Scalars['String']['input'];
};


export type MutationSaveSettingsArgs = {
  settings: NewGlobalSettings;
};


export type MutationUnlikeAlbumArgs = {
  id: Scalars['String']['input'];
};


export type MutationUnlikeTrackArgs = {
  id: Scalars['String']['input'];
};


export type MutationUpdateSavedPlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  folderId?: InputMaybe<Scalars['String']['input']>;
  id: Scalars['String']['input'];
  image?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
};


export type MutationUpdateSmartPlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  folderId?: InputMaybe<Scalars['String']['input']>;
  id: Scalars['String']['input'];
  image?: InputMaybe<Scalars['String']['input']>;
  name: Scalars['String']['input'];
  rules: Scalars['String']['input'];
};

export type NewGlobalSettings = {
  balance?: InputMaybe<Scalars['Int']['input']>;
  bass?: InputMaybe<Scalars['Int']['input']>;
  bassCutoff?: InputMaybe<Scalars['Int']['input']>;
  channelConfig?: InputMaybe<Scalars['Int']['input']>;
  compressorSettings?: InputMaybe<CompressorSettingsInput>;
  crossfade?: InputMaybe<Scalars['Int']['input']>;
  eqBandSettings?: InputMaybe<Array<EqBandSettingInput>>;
  eqEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  fadeInDelay?: InputMaybe<Scalars['Int']['input']>;
  fadeInDuration?: InputMaybe<Scalars['Int']['input']>;
  fadeOnStop?: InputMaybe<Scalars['Boolean']['input']>;
  fadeOutDelay?: InputMaybe<Scalars['Int']['input']>;
  fadeOutDuration?: InputMaybe<Scalars['Int']['input']>;
  fadeOutMixmode?: InputMaybe<Scalars['Int']['input']>;
  musicDir?: InputMaybe<Scalars['String']['input']>;
  partyMode?: InputMaybe<Scalars['Boolean']['input']>;
  playerName?: InputMaybe<Scalars['String']['input']>;
  playlistShuffle?: InputMaybe<Scalars['Boolean']['input']>;
  repeatMode?: InputMaybe<Scalars['Int']['input']>;
  replaygainSettings?: InputMaybe<ReplaygainSettingsInput>;
  stereoWidth?: InputMaybe<Scalars['Int']['input']>;
  stereoswMode?: InputMaybe<Scalars['Int']['input']>;
  surroundBalance?: InputMaybe<Scalars['Int']['input']>;
  surroundEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  surroundFx1?: InputMaybe<Scalars['Int']['input']>;
  surroundFx2?: InputMaybe<Scalars['Int']['input']>;
  treble?: InputMaybe<Scalars['Int']['input']>;
  trebleCutoff?: InputMaybe<Scalars['Int']['input']>;
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
  device?: Maybe<Device>;
  devices: Array<Device>;
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
  playlistFolders: Array<SavedPlaylistFolder>;
  playlistGetCurrent: Playlist;
  rockboxVersion: Scalars['String']['output'];
  savedPlaylist?: Maybe<SavedPlaylist>;
  savedPlaylistTrackIds: Array<Scalars['String']['output']>;
  savedPlaylistTracks: Array<Track>;
  savedPlaylists: Array<SavedPlaylist>;
  search: SearchResults;
  smartPlaylist?: Maybe<SmartPlaylist>;
  smartPlaylistTrackIds: Array<Scalars['String']['output']>;
  smartPlaylistTracks: Array<Track>;
  smartPlaylists: Array<SmartPlaylist>;
  soundCurrent: Scalars['String']['output'];
  soundDefault: Scalars['String']['output'];
  soundVal2Phys: Scalars['String']['output'];
  status: Scalars['Int']['output'];
  track?: Maybe<Track>;
  trackStats?: Maybe<TrackStats>;
  tracks: Array<Track>;
  treeGetEntries: Array<Entry>;
};


export type QueryAlbumArgs = {
  id: Scalars['String']['input'];
};


export type QueryArtistArgs = {
  id: Scalars['String']['input'];
};


export type QueryDeviceArgs = {
  id: Scalars['String']['input'];
};


export type QuerySavedPlaylistArgs = {
  id: Scalars['String']['input'];
};


export type QuerySavedPlaylistTrackIdsArgs = {
  playlistId: Scalars['String']['input'];
};


export type QuerySavedPlaylistTracksArgs = {
  playlistId: Scalars['String']['input'];
};


export type QuerySavedPlaylistsArgs = {
  folderId?: InputMaybe<Scalars['String']['input']>;
};


export type QuerySearchArgs = {
  term: Scalars['String']['input'];
};


export type QuerySmartPlaylistArgs = {
  id: Scalars['String']['input'];
};


export type QuerySmartPlaylistTrackIdsArgs = {
  id: Scalars['String']['input'];
};


export type QuerySmartPlaylistTracksArgs = {
  id: Scalars['String']['input'];
};


export type QueryTrackArgs = {
  id: Scalars['String']['input'];
};


export type QueryTrackStatsArgs = {
  trackId: Scalars['String']['input'];
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

export type ReplaygainSettingsInput = {
  noclip: Scalars['Boolean']['input'];
  preamp: Scalars['Int']['input'];
  type: Scalars['Int']['input'];
};

export type SavedPlaylist = {
  __typename?: 'SavedPlaylist';
  createdAt: Scalars['Int']['output'];
  description?: Maybe<Scalars['String']['output']>;
  folderId?: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  image?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  trackCount: Scalars['Int']['output'];
  updatedAt: Scalars['Int']['output'];
};

export type SavedPlaylistFolder = {
  __typename?: 'SavedPlaylistFolder';
  createdAt: Scalars['Int']['output'];
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  updatedAt: Scalars['Int']['output'];
};

export type SearchResults = {
  __typename?: 'SearchResults';
  albums: Array<Album>;
  artists: Array<Artist>;
  likedAlbums: Array<Album>;
  likedTracks: Array<Track>;
  tracks: Array<Track>;
};

export type SmartPlaylist = {
  __typename?: 'SmartPlaylist';
  createdAt: Scalars['Int']['output'];
  description?: Maybe<Scalars['String']['output']>;
  folderId?: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  image?: Maybe<Scalars['String']['output']>;
  isSystem: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  rules: Scalars['String']['output'];
  updatedAt: Scalars['Int']['output'];
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

export type TrackStats = {
  __typename?: 'TrackStats';
  lastPlayed?: Maybe<Scalars['Int']['output']>;
  lastSkipped?: Maybe<Scalars['Int']['output']>;
  playCount: Scalars['Int']['output'];
  skipCount: Scalars['Int']['output'];
  trackId: Scalars['String']['output'];
  updatedAt: Scalars['Int']['output'];
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
  musicDir: Scalars['String']['output'];
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
  surroundFx2: Scalars['Int']['output'];
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


export type GetEntriesQuery = { __typename?: 'Query', treeGetEntries: Array<{ __typename?: 'Entry', name: string, attr: number, timeWrite: number, displayName?: string | null }> };

export type ConnectToDeviceMutationVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type ConnectToDeviceMutation = { __typename?: 'Mutation', connect: boolean };

export type DisconnectFromDeviceMutationVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type DisconnectFromDeviceMutation = { __typename?: 'Mutation', disconnect: boolean };

export type GetDevicesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetDevicesQuery = { __typename?: 'Query', devices: Array<{ __typename?: 'Device', id: string, name: string, app: string, ip: string, host: string, port: number, isCastDevice: boolean, service: string, isConnected: boolean, isCurrentDevice: boolean }> };

export type GetDeviceQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetDeviceQuery = { __typename?: 'Query', device?: { __typename?: 'Device', id: string, name: string, app: string, ip: string, host: string, port: number, isCastDevice: boolean, service: string, isConnected: boolean, isCurrentDevice: boolean } | null };

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


export type GetArtistsQuery = { __typename?: 'Query', artists: Array<{ __typename?: 'Artist', id: string, name: string, image?: string | null }> };

export type GetArtistQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetArtistQuery = { __typename?: 'Query', artist?: { __typename?: 'Artist', id: string, name: string, image?: string | null, albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string }>, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> } | null };

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

export type SearchQueryVariables = Exact<{
  term: Scalars['String']['input'];
}>;


export type SearchQuery = { __typename?: 'Query', search: { __typename?: 'SearchResults', tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArtist: string, path: string, albumArt?: string | null, length: number, composer: string, comment: string, albumId?: string | null, artistId?: string | null }>, albums: Array<{ __typename?: 'Album', id: string, title: string, year: number, yearString: string, albumArt?: string | null, artist: string, artistId: string }>, artists: Array<{ __typename?: 'Artist', id: string, name: string, image?: string | null }>, likedTracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArtist: string, path: string, albumArt?: string | null, length: number, composer: string, comment: string, albumId?: string | null, artistId?: string | null }>, likedAlbums: Array<{ __typename?: 'Album', id: string, title: string, albumArt?: string | null, artist: string, artistId: string, year: number }> } };

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
  position?: InputMaybe<Scalars['Int']['input']>;
}>;


export type PlayAlbumMutation = { __typename?: 'Mutation', playAlbum: number };

export type PlayArtistTracksMutationVariables = Exact<{
  artistId: Scalars['String']['input'];
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
  position?: InputMaybe<Scalars['Int']['input']>;
}>;


export type PlayArtistTracksMutation = { __typename?: 'Mutation', playArtistTracks: number };

export type PlayDirectoryMutationVariables = Exact<{
  path: Scalars['String']['input'];
  recurse?: InputMaybe<Scalars['Boolean']['input']>;
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
  position?: InputMaybe<Scalars['Int']['input']>;
}>;


export type PlayDirectoryMutation = { __typename?: 'Mutation', playDirectory: number };

export type PlayTrackMutationVariables = Exact<{
  path: Scalars['String']['input'];
}>;


export type PlayTrackMutation = { __typename?: 'Mutation', playTrack: number };

export type PlayLikedTracksMutationVariables = Exact<{
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
  position?: InputMaybe<Scalars['Int']['input']>;
}>;


export type PlayLikedTracksMutation = { __typename?: 'Mutation', playLikedTracks: number };

export type PlayAllTracksMutationVariables = Exact<{
  shuffle?: InputMaybe<Scalars['Boolean']['input']>;
  position?: InputMaybe<Scalars['Int']['input']>;
}>;


export type PlayAllTracksMutation = { __typename?: 'Mutation', playAllTracks: number };

export type SeekMutationVariables = Exact<{
  elapsed: Scalars['Int']['input'];
  offset: Scalars['Int']['input'];
}>;


export type SeekMutation = { __typename?: 'Mutation', play: number };

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

export type ShufflePlaylistMutationVariables = Exact<{ [key: string]: never; }>;


export type ShufflePlaylistMutation = { __typename?: 'Mutation', shufflePlaylist: number };

export type GetCurrentPlaylistQueryVariables = Exact<{ [key: string]: never; }>;


export type GetCurrentPlaylistQuery = { __typename?: 'Query', playlistGetCurrent: { __typename?: 'Playlist', index: number, amount: number, maxPlaylistSize: number, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, path: string, album: string, length: number }> } };

export type PlaylistChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type PlaylistChangedSubscription = { __typename?: 'Subscription', playlistChanged: { __typename?: 'Playlist', index: number, amount: number, maxPlaylistSize: number, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, path: string }> } };

export type CreateSavedPlaylistMutationVariables = Exact<{
  name: Scalars['String']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
  trackIds?: InputMaybe<Array<Scalars['String']['input']> | Scalars['String']['input']>;
}>;


export type CreateSavedPlaylistMutation = { __typename?: 'Mutation', createSavedPlaylist: { __typename?: 'SavedPlaylist', id: string, name: string, description?: string | null, trackCount: number } };

export type UpdateSavedPlaylistMutationVariables = Exact<{
  id: Scalars['String']['input'];
  name: Scalars['String']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
}>;


export type UpdateSavedPlaylistMutation = { __typename?: 'Mutation', updateSavedPlaylist: boolean };

export type DeleteSavedPlaylistMutationVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type DeleteSavedPlaylistMutation = { __typename?: 'Mutation', deleteSavedPlaylist: boolean };

export type AddTracksToSavedPlaylistMutationVariables = Exact<{
  playlistId: Scalars['String']['input'];
  trackIds: Array<Scalars['String']['input']> | Scalars['String']['input'];
}>;


export type AddTracksToSavedPlaylistMutation = { __typename?: 'Mutation', addTracksToSavedPlaylist: boolean };

export type RemoveTrackFromSavedPlaylistMutationVariables = Exact<{
  playlistId: Scalars['String']['input'];
  trackId: Scalars['String']['input'];
}>;


export type RemoveTrackFromSavedPlaylistMutation = { __typename?: 'Mutation', removeTrackFromSavedPlaylist: boolean };

export type PlaySavedPlaylistMutationVariables = Exact<{
  playlistId: Scalars['String']['input'];
}>;


export type PlaySavedPlaylistMutation = { __typename?: 'Mutation', playSavedPlaylist: boolean };

export type GetSavedPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetSavedPlaylistsQuery = { __typename?: 'Query', savedPlaylists: Array<{ __typename?: 'SavedPlaylist', id: string, name: string, description?: string | null, image?: string | null, trackCount: number, createdAt: number, updatedAt: number }> };

export type GetSavedPlaylistQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetSavedPlaylistQuery = { __typename?: 'Query', savedPlaylist?: { __typename?: 'SavedPlaylist', id: string, name: string, description?: string | null, image?: string | null, trackCount: number, createdAt: number, updatedAt: number } | null };

export type GetSavedPlaylistTracksQueryVariables = Exact<{
  playlistId: Scalars['String']['input'];
}>;


export type GetSavedPlaylistTracksQuery = { __typename?: 'Query', savedPlaylistTracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, path: string, length: number, tracknum: number }> };

export type SaveSettingsMutationVariables = Exact<{
  settings: NewGlobalSettings;
}>;


export type SaveSettingsMutation = { __typename?: 'Mutation', saveSettings: boolean };

export type GetGlobalSettingsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetGlobalSettingsQuery = { __typename?: 'Query', globalSettings: { __typename?: 'UserSettings', musicDir: string, volume: number, playlistShuffle: boolean, repeatMode: number, bass: number, bassCutoff: number, treble: number, trebleCutoff: number, crossfade: number, fadeOnStop: boolean, crossfadeFadeInDelay: number, crossfadeFadeInDuration: number, crossfadeFadeOutDelay: number, crossfadeFadeOutDuration: number, crossfadeFadeOutMixmode: number, balance: number, stereoWidth: number, stereoswMode: number, surroundEnabled: number, surroundBalance: number, surroundFx1: number, surroundFx2: number, partyMode: boolean, ditheringEnabled: boolean, channelConfig: number, playerName: string, eqEnabled: boolean, eqBandSettings: Array<{ __typename?: 'EqBandSetting', q: number, cutoff: number, gain: number }>, replaygainSettings: { __typename?: 'ReplaygainSettings', noclip: boolean, type: number, preamp: number } } };

export type PlaySmartPlaylistMutationVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type PlaySmartPlaylistMutation = { __typename?: 'Mutation', playSmartPlaylist: boolean };

export type GetSmartPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetSmartPlaylistsQuery = { __typename?: 'Query', smartPlaylists: Array<{ __typename?: 'SmartPlaylist', id: string, name: string, description?: string | null, image?: string | null, isSystem: boolean, createdAt: number, updatedAt: number }> };

export type GetSmartPlaylistQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetSmartPlaylistQuery = { __typename?: 'Query', smartPlaylist?: { __typename?: 'SmartPlaylist', id: string, name: string, description?: string | null, image?: string | null, isSystem: boolean, createdAt: number, updatedAt: number } | null };

export type GetSmartPlaylistTracksQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetSmartPlaylistTracksQuery = { __typename?: 'Query', smartPlaylistTracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArt?: string | null, artistId?: string | null, albumId?: string | null, path: string, length: number, tracknum: number }> };

export type AdjustVolumeMutationVariables = Exact<{
  steps: Scalars['Int']['input'];
}>;


export type AdjustVolumeMutation = { __typename?: 'Mutation', adjustVolume: number };

export type GetRockboxVersionQueryVariables = Exact<{ [key: string]: never; }>;


export type GetRockboxVersionQuery = { __typename?: 'Query', rockboxVersion: string };

export type GetGlobalStatusQueryVariables = Exact<{ [key: string]: never; }>;


export type GetGlobalStatusQuery = { __typename?: 'Query', globalStatus: { __typename?: 'SystemStatus', resumeIndex: number, resumeCrc32: number, resumeOffset: number, resumeElapsed: number } };



export const GetEntriesDocument = new TypedDocumentString(`
    query GetEntries($path: String) {
  treeGetEntries(path: $path) {
    name
    attr
    timeWrite
    displayName
  }
}
    `);

export const useGetEntriesQuery = <
      TData = GetEntriesQuery,
      TError = unknown
    >(
      variables?: GetEntriesQueryVariables,
      options?: Omit<UseQueryOptions<GetEntriesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetEntriesQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetEntriesQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetEntries'] : ['GetEntries', variables],
    queryFn: fetchData<GetEntriesQuery, GetEntriesQueryVariables>(GetEntriesDocument, variables),
    ...options
  }
    )};

useGetEntriesQuery.document = GetEntriesDocument;

useGetEntriesQuery.getKey = (variables?: GetEntriesQueryVariables) => variables === undefined ? ['GetEntries'] : ['GetEntries', variables];

export const ConnectToDeviceDocument = new TypedDocumentString(`
    mutation ConnectToDevice($id: String!) {
  connect(id: $id)
}
    `);

export const useConnectToDeviceMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ConnectToDeviceMutation, TError, ConnectToDeviceMutationVariables, TContext>) => {
    
    return useMutation<ConnectToDeviceMutation, TError, ConnectToDeviceMutationVariables, TContext>(
      {
    mutationKey: ['ConnectToDevice'],
    mutationFn: (variables?: ConnectToDeviceMutationVariables) => fetchData<ConnectToDeviceMutation, ConnectToDeviceMutationVariables>(ConnectToDeviceDocument, variables)(),
    ...options
  }
    )};

useConnectToDeviceMutation.getKey = () => ['ConnectToDevice'];

export const DisconnectFromDeviceDocument = new TypedDocumentString(`
    mutation DisconnectFromDevice($id: String!) {
  disconnect(id: $id)
}
    `);

export const useDisconnectFromDeviceMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DisconnectFromDeviceMutation, TError, DisconnectFromDeviceMutationVariables, TContext>) => {
    
    return useMutation<DisconnectFromDeviceMutation, TError, DisconnectFromDeviceMutationVariables, TContext>(
      {
    mutationKey: ['DisconnectFromDevice'],
    mutationFn: (variables?: DisconnectFromDeviceMutationVariables) => fetchData<DisconnectFromDeviceMutation, DisconnectFromDeviceMutationVariables>(DisconnectFromDeviceDocument, variables)(),
    ...options
  }
    )};

useDisconnectFromDeviceMutation.getKey = () => ['DisconnectFromDevice'];

export const GetDevicesDocument = new TypedDocumentString(`
    query GetDevices {
  devices {
    id
    name
    app
    ip
    host
    port
    isCastDevice
    service
    isConnected
    isCurrentDevice
  }
}
    `);

export const useGetDevicesQuery = <
      TData = GetDevicesQuery,
      TError = unknown
    >(
      variables?: GetDevicesQueryVariables,
      options?: Omit<UseQueryOptions<GetDevicesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetDevicesQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetDevicesQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetDevices'] : ['GetDevices', variables],
    queryFn: fetchData<GetDevicesQuery, GetDevicesQueryVariables>(GetDevicesDocument, variables),
    ...options
  }
    )};

useGetDevicesQuery.document = GetDevicesDocument;

useGetDevicesQuery.getKey = (variables?: GetDevicesQueryVariables) => variables === undefined ? ['GetDevices'] : ['GetDevices', variables];

export const GetDeviceDocument = new TypedDocumentString(`
    query GetDevice($id: String!) {
  device(id: $id) {
    id
    name
    app
    ip
    host
    port
    isCastDevice
    service
    isConnected
    isCurrentDevice
  }
}
    `);

export const useGetDeviceQuery = <
      TData = GetDeviceQuery,
      TError = unknown
    >(
      variables: GetDeviceQueryVariables,
      options?: Omit<UseQueryOptions<GetDeviceQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetDeviceQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetDeviceQuery, TError, TData>(
      {
    queryKey: ['GetDevice', variables],
    queryFn: fetchData<GetDeviceQuery, GetDeviceQueryVariables>(GetDeviceDocument, variables),
    ...options
  }
    )};

useGetDeviceQuery.document = GetDeviceDocument;

useGetDeviceQuery.getKey = (variables: GetDeviceQueryVariables) => ['GetDevice', variables];

export const LikeTrackDocument = new TypedDocumentString(`
    mutation LikeTrack($trackId: String!) {
  likeTrack(id: $trackId)
}
    `);

export const useLikeTrackMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<LikeTrackMutation, TError, LikeTrackMutationVariables, TContext>) => {
    
    return useMutation<LikeTrackMutation, TError, LikeTrackMutationVariables, TContext>(
      {
    mutationKey: ['LikeTrack'],
    mutationFn: (variables?: LikeTrackMutationVariables) => fetchData<LikeTrackMutation, LikeTrackMutationVariables>(LikeTrackDocument, variables)(),
    ...options
  }
    )};

useLikeTrackMutation.getKey = () => ['LikeTrack'];

export const UnlikeTrackDocument = new TypedDocumentString(`
    mutation UnlikeTrack($trackId: String!) {
  unlikeTrack(id: $trackId)
}
    `);

export const useUnlikeTrackMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<UnlikeTrackMutation, TError, UnlikeTrackMutationVariables, TContext>) => {
    
    return useMutation<UnlikeTrackMutation, TError, UnlikeTrackMutationVariables, TContext>(
      {
    mutationKey: ['UnlikeTrack'],
    mutationFn: (variables?: UnlikeTrackMutationVariables) => fetchData<UnlikeTrackMutation, UnlikeTrackMutationVariables>(UnlikeTrackDocument, variables)(),
    ...options
  }
    )};

useUnlikeTrackMutation.getKey = () => ['UnlikeTrack'];

export const LikeAlbumDocument = new TypedDocumentString(`
    mutation LikeAlbum($albumId: String!) {
  likeAlbum(id: $albumId)
}
    `);

export const useLikeAlbumMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<LikeAlbumMutation, TError, LikeAlbumMutationVariables, TContext>) => {
    
    return useMutation<LikeAlbumMutation, TError, LikeAlbumMutationVariables, TContext>(
      {
    mutationKey: ['LikeAlbum'],
    mutationFn: (variables?: LikeAlbumMutationVariables) => fetchData<LikeAlbumMutation, LikeAlbumMutationVariables>(LikeAlbumDocument, variables)(),
    ...options
  }
    )};

useLikeAlbumMutation.getKey = () => ['LikeAlbum'];

export const UnlikeAlbumDocument = new TypedDocumentString(`
    mutation UnlikeAlbum($albumId: String!) {
  unlikeAlbum(id: $albumId)
}
    `);

export const useUnlikeAlbumMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<UnlikeAlbumMutation, TError, UnlikeAlbumMutationVariables, TContext>) => {
    
    return useMutation<UnlikeAlbumMutation, TError, UnlikeAlbumMutationVariables, TContext>(
      {
    mutationKey: ['UnlikeAlbum'],
    mutationFn: (variables?: UnlikeAlbumMutationVariables) => fetchData<UnlikeAlbumMutation, UnlikeAlbumMutationVariables>(UnlikeAlbumDocument, variables)(),
    ...options
  }
    )};

useUnlikeAlbumMutation.getKey = () => ['UnlikeAlbum'];

export const GetAlbumsDocument = new TypedDocumentString(`
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
    `);

export const useGetAlbumsQuery = <
      TData = GetAlbumsQuery,
      TError = unknown
    >(
      variables?: GetAlbumsQueryVariables,
      options?: Omit<UseQueryOptions<GetAlbumsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetAlbumsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetAlbumsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetAlbums'] : ['GetAlbums', variables],
    queryFn: fetchData<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, variables),
    ...options
  }
    )};

useGetAlbumsQuery.document = GetAlbumsDocument;

useGetAlbumsQuery.getKey = (variables?: GetAlbumsQueryVariables) => variables === undefined ? ['GetAlbums'] : ['GetAlbums', variables];

export const GetArtistsDocument = new TypedDocumentString(`
    query GetArtists {
  artists {
    id
    name
    image
  }
}
    `);

export const useGetArtistsQuery = <
      TData = GetArtistsQuery,
      TError = unknown
    >(
      variables?: GetArtistsQueryVariables,
      options?: Omit<UseQueryOptions<GetArtistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetArtistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetArtistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetArtists'] : ['GetArtists', variables],
    queryFn: fetchData<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, variables),
    ...options
  }
    )};

useGetArtistsQuery.document = GetArtistsDocument;

useGetArtistsQuery.getKey = (variables?: GetArtistsQueryVariables) => variables === undefined ? ['GetArtists'] : ['GetArtists', variables];

export const GetArtistDocument = new TypedDocumentString(`
    query GetArtist($id: String!) {
  artist(id: $id) {
    id
    name
    image
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
    `);

export const useGetArtistQuery = <
      TData = GetArtistQuery,
      TError = unknown
    >(
      variables: GetArtistQueryVariables,
      options?: Omit<UseQueryOptions<GetArtistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetArtistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetArtistQuery, TError, TData>(
      {
    queryKey: ['GetArtist', variables],
    queryFn: fetchData<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, variables),
    ...options
  }
    )};

useGetArtistQuery.document = GetArtistDocument;

useGetArtistQuery.getKey = (variables: GetArtistQueryVariables) => ['GetArtist', variables];

export const TracksDocument = new TypedDocumentString(`
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
    `);

export const useTracksQuery = <
      TData = TracksQuery,
      TError = unknown
    >(
      variables?: TracksQueryVariables,
      options?: Omit<UseQueryOptions<TracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<TracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<TracksQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['Tracks'] : ['Tracks', variables],
    queryFn: fetchData<TracksQuery, TracksQueryVariables>(TracksDocument, variables),
    ...options
  }
    )};

useTracksQuery.document = TracksDocument;

useTracksQuery.getKey = (variables?: TracksQueryVariables) => variables === undefined ? ['Tracks'] : ['Tracks', variables];

export const GetAlbumDocument = new TypedDocumentString(`
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
    `);

export const useGetAlbumQuery = <
      TData = GetAlbumQuery,
      TError = unknown
    >(
      variables: GetAlbumQueryVariables,
      options?: Omit<UseQueryOptions<GetAlbumQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetAlbumQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetAlbumQuery, TError, TData>(
      {
    queryKey: ['GetAlbum', variables],
    queryFn: fetchData<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, variables),
    ...options
  }
    )};

useGetAlbumQuery.document = GetAlbumDocument;

useGetAlbumQuery.getKey = (variables: GetAlbumQueryVariables) => ['GetAlbum', variables];

export const GetLikedTracksDocument = new TypedDocumentString(`
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
    `);

export const useGetLikedTracksQuery = <
      TData = GetLikedTracksQuery,
      TError = unknown
    >(
      variables?: GetLikedTracksQueryVariables,
      options?: Omit<UseQueryOptions<GetLikedTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetLikedTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetLikedTracksQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetLikedTracks'] : ['GetLikedTracks', variables],
    queryFn: fetchData<GetLikedTracksQuery, GetLikedTracksQueryVariables>(GetLikedTracksDocument, variables),
    ...options
  }
    )};

useGetLikedTracksQuery.document = GetLikedTracksDocument;

useGetLikedTracksQuery.getKey = (variables?: GetLikedTracksQueryVariables) => variables === undefined ? ['GetLikedTracks'] : ['GetLikedTracks', variables];

export const GetLikedAlbumsDocument = new TypedDocumentString(`
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
    `);

export const useGetLikedAlbumsQuery = <
      TData = GetLikedAlbumsQuery,
      TError = unknown
    >(
      variables?: GetLikedAlbumsQueryVariables,
      options?: Omit<UseQueryOptions<GetLikedAlbumsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetLikedAlbumsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetLikedAlbumsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetLikedAlbums'] : ['GetLikedAlbums', variables],
    queryFn: fetchData<GetLikedAlbumsQuery, GetLikedAlbumsQueryVariables>(GetLikedAlbumsDocument, variables),
    ...options
  }
    )};

useGetLikedAlbumsQuery.document = GetLikedAlbumsDocument;

useGetLikedAlbumsQuery.getKey = (variables?: GetLikedAlbumsQueryVariables) => variables === undefined ? ['GetLikedAlbums'] : ['GetLikedAlbums', variables];

export const SearchDocument = new TypedDocumentString(`
    query Search($term: String!) {
  search(term: $term) {
    tracks {
      id
      title
      artist
      album
      albumArtist
      path
      albumArt
      length
      composer
      comment
      albumId
      artistId
    }
    albums {
      id
      title
      year
      yearString
      albumArt
      artist
      artistId
    }
    artists {
      id
      name
      image
    }
    likedTracks {
      id
      title
      artist
      album
      albumArtist
      path
      albumArt
      length
      composer
      comment
      albumId
      artistId
    }
    likedAlbums {
      id
      title
      albumArt
      artist
      artistId
      year
    }
  }
}
    `);

export const useSearchQuery = <
      TData = SearchQuery,
      TError = unknown
    >(
      variables: SearchQueryVariables,
      options?: Omit<UseQueryOptions<SearchQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<SearchQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<SearchQuery, TError, TData>(
      {
    queryKey: ['Search', variables],
    queryFn: fetchData<SearchQuery, SearchQueryVariables>(SearchDocument, variables),
    ...options
  }
    )};

useSearchQuery.document = SearchDocument;

useSearchQuery.getKey = (variables: SearchQueryVariables) => ['Search', variables];

export const PlayDocument = new TypedDocumentString(`
    mutation Play($elapsed: Int!, $offset: Int!) {
  play(elapsed: $elapsed, offset: $offset)
}
    `);

export const usePlayMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayMutation, TError, PlayMutationVariables, TContext>) => {
    
    return useMutation<PlayMutation, TError, PlayMutationVariables, TContext>(
      {
    mutationKey: ['Play'],
    mutationFn: (variables?: PlayMutationVariables) => fetchData<PlayMutation, PlayMutationVariables>(PlayDocument, variables)(),
    ...options
  }
    )};

usePlayMutation.getKey = () => ['Play'];

export const PauseDocument = new TypedDocumentString(`
    mutation Pause {
  pause
}
    `);

export const usePauseMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PauseMutation, TError, PauseMutationVariables, TContext>) => {
    
    return useMutation<PauseMutation, TError, PauseMutationVariables, TContext>(
      {
    mutationKey: ['Pause'],
    mutationFn: (variables?: PauseMutationVariables) => fetchData<PauseMutation, PauseMutationVariables>(PauseDocument, variables)(),
    ...options
  }
    )};

usePauseMutation.getKey = () => ['Pause'];

export const ResumeDocument = new TypedDocumentString(`
    mutation Resume {
  resume
}
    `);

export const useResumeMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ResumeMutation, TError, ResumeMutationVariables, TContext>) => {
    
    return useMutation<ResumeMutation, TError, ResumeMutationVariables, TContext>(
      {
    mutationKey: ['Resume'],
    mutationFn: (variables?: ResumeMutationVariables) => fetchData<ResumeMutation, ResumeMutationVariables>(ResumeDocument, variables)(),
    ...options
  }
    )};

useResumeMutation.getKey = () => ['Resume'];

export const PreviousDocument = new TypedDocumentString(`
    mutation Previous {
  previous
}
    `);

export const usePreviousMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PreviousMutation, TError, PreviousMutationVariables, TContext>) => {
    
    return useMutation<PreviousMutation, TError, PreviousMutationVariables, TContext>(
      {
    mutationKey: ['Previous'],
    mutationFn: (variables?: PreviousMutationVariables) => fetchData<PreviousMutation, PreviousMutationVariables>(PreviousDocument, variables)(),
    ...options
  }
    )};

usePreviousMutation.getKey = () => ['Previous'];

export const NextDocument = new TypedDocumentString(`
    mutation Next {
  next
}
    `);

export const useNextMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<NextMutation, TError, NextMutationVariables, TContext>) => {
    
    return useMutation<NextMutation, TError, NextMutationVariables, TContext>(
      {
    mutationKey: ['Next'],
    mutationFn: (variables?: NextMutationVariables) => fetchData<NextMutation, NextMutationVariables>(NextDocument, variables)(),
    ...options
  }
    )};

useNextMutation.getKey = () => ['Next'];

export const PlayAlbumDocument = new TypedDocumentString(`
    mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) {
  playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position)
}
    `);

export const usePlayAlbumMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayAlbumMutation, TError, PlayAlbumMutationVariables, TContext>) => {
    
    return useMutation<PlayAlbumMutation, TError, PlayAlbumMutationVariables, TContext>(
      {
    mutationKey: ['PlayAlbum'],
    mutationFn: (variables?: PlayAlbumMutationVariables) => fetchData<PlayAlbumMutation, PlayAlbumMutationVariables>(PlayAlbumDocument, variables)(),
    ...options
  }
    )};

usePlayAlbumMutation.getKey = () => ['PlayAlbum'];

export const PlayArtistTracksDocument = new TypedDocumentString(`
    mutation PlayArtistTracks($artistId: String!, $shuffle: Boolean, $position: Int) {
  playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position)
}
    `);

export const usePlayArtistTracksMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayArtistTracksMutation, TError, PlayArtistTracksMutationVariables, TContext>) => {
    
    return useMutation<PlayArtistTracksMutation, TError, PlayArtistTracksMutationVariables, TContext>(
      {
    mutationKey: ['PlayArtistTracks'],
    mutationFn: (variables?: PlayArtistTracksMutationVariables) => fetchData<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>(PlayArtistTracksDocument, variables)(),
    ...options
  }
    )};

usePlayArtistTracksMutation.getKey = () => ['PlayArtistTracks'];

export const PlayDirectoryDocument = new TypedDocumentString(`
    mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int) {
  playDirectory(
    path: $path
    recurse: $recurse
    shuffle: $shuffle
    position: $position
  )
}
    `);

export const usePlayDirectoryMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayDirectoryMutation, TError, PlayDirectoryMutationVariables, TContext>) => {
    
    return useMutation<PlayDirectoryMutation, TError, PlayDirectoryMutationVariables, TContext>(
      {
    mutationKey: ['PlayDirectory'],
    mutationFn: (variables?: PlayDirectoryMutationVariables) => fetchData<PlayDirectoryMutation, PlayDirectoryMutationVariables>(PlayDirectoryDocument, variables)(),
    ...options
  }
    )};

usePlayDirectoryMutation.getKey = () => ['PlayDirectory'];

export const PlayTrackDocument = new TypedDocumentString(`
    mutation PlayTrack($path: String!) {
  playTrack(path: $path)
}
    `);

export const usePlayTrackMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayTrackMutation, TError, PlayTrackMutationVariables, TContext>) => {
    
    return useMutation<PlayTrackMutation, TError, PlayTrackMutationVariables, TContext>(
      {
    mutationKey: ['PlayTrack'],
    mutationFn: (variables?: PlayTrackMutationVariables) => fetchData<PlayTrackMutation, PlayTrackMutationVariables>(PlayTrackDocument, variables)(),
    ...options
  }
    )};

usePlayTrackMutation.getKey = () => ['PlayTrack'];

export const PlayLikedTracksDocument = new TypedDocumentString(`
    mutation PlayLikedTracks($shuffle: Boolean, $position: Int) {
  playLikedTracks(shuffle: $shuffle, position: $position)
}
    `);

export const usePlayLikedTracksMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayLikedTracksMutation, TError, PlayLikedTracksMutationVariables, TContext>) => {
    
    return useMutation<PlayLikedTracksMutation, TError, PlayLikedTracksMutationVariables, TContext>(
      {
    mutationKey: ['PlayLikedTracks'],
    mutationFn: (variables?: PlayLikedTracksMutationVariables) => fetchData<PlayLikedTracksMutation, PlayLikedTracksMutationVariables>(PlayLikedTracksDocument, variables)(),
    ...options
  }
    )};

usePlayLikedTracksMutation.getKey = () => ['PlayLikedTracks'];

export const PlayAllTracksDocument = new TypedDocumentString(`
    mutation PlayAllTracks($shuffle: Boolean, $position: Int) {
  playAllTracks(shuffle: $shuffle, position: $position)
}
    `);

export const usePlayAllTracksMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayAllTracksMutation, TError, PlayAllTracksMutationVariables, TContext>) => {
    
    return useMutation<PlayAllTracksMutation, TError, PlayAllTracksMutationVariables, TContext>(
      {
    mutationKey: ['PlayAllTracks'],
    mutationFn: (variables?: PlayAllTracksMutationVariables) => fetchData<PlayAllTracksMutation, PlayAllTracksMutationVariables>(PlayAllTracksDocument, variables)(),
    ...options
  }
    )};

usePlayAllTracksMutation.getKey = () => ['PlayAllTracks'];

export const SeekDocument = new TypedDocumentString(`
    mutation Seek($elapsed: Int!, $offset: Int!) {
  play(elapsed: $elapsed, offset: $offset)
}
    `);

export const useSeekMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<SeekMutation, TError, SeekMutationVariables, TContext>) => {
    
    return useMutation<SeekMutation, TError, SeekMutationVariables, TContext>(
      {
    mutationKey: ['Seek'],
    mutationFn: (variables?: SeekMutationVariables) => fetchData<SeekMutation, SeekMutationVariables>(SeekDocument, variables)(),
    ...options
  }
    )};

useSeekMutation.getKey = () => ['Seek'];

export const GetCurrentTrackDocument = new TypedDocumentString(`
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
    `);

export const useGetCurrentTrackQuery = <
      TData = GetCurrentTrackQuery,
      TError = unknown
    >(
      variables?: GetCurrentTrackQueryVariables,
      options?: Omit<UseQueryOptions<GetCurrentTrackQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetCurrentTrackQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetCurrentTrackQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetCurrentTrack'] : ['GetCurrentTrack', variables],
    queryFn: fetchData<GetCurrentTrackQuery, GetCurrentTrackQueryVariables>(GetCurrentTrackDocument, variables),
    ...options
  }
    )};

useGetCurrentTrackQuery.document = GetCurrentTrackDocument;

useGetCurrentTrackQuery.getKey = (variables?: GetCurrentTrackQueryVariables) => variables === undefined ? ['GetCurrentTrack'] : ['GetCurrentTrack', variables];

export const GetNextTrackDocument = new TypedDocumentString(`
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
    `);

export const useGetNextTrackQuery = <
      TData = GetNextTrackQuery,
      TError = unknown
    >(
      variables?: GetNextTrackQueryVariables,
      options?: Omit<UseQueryOptions<GetNextTrackQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetNextTrackQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetNextTrackQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetNextTrack'] : ['GetNextTrack', variables],
    queryFn: fetchData<GetNextTrackQuery, GetNextTrackQueryVariables>(GetNextTrackDocument, variables),
    ...options
  }
    )};

useGetNextTrackQuery.document = GetNextTrackDocument;

useGetNextTrackQuery.getKey = (variables?: GetNextTrackQueryVariables) => variables === undefined ? ['GetNextTrack'] : ['GetNextTrack', variables];

export const GetPlaybackStatusDocument = new TypedDocumentString(`
    query GetPlaybackStatus {
  status
}
    `);

export const useGetPlaybackStatusQuery = <
      TData = GetPlaybackStatusQuery,
      TError = unknown
    >(
      variables?: GetPlaybackStatusQueryVariables,
      options?: Omit<UseQueryOptions<GetPlaybackStatusQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetPlaybackStatusQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetPlaybackStatusQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetPlaybackStatus'] : ['GetPlaybackStatus', variables],
    queryFn: fetchData<GetPlaybackStatusQuery, GetPlaybackStatusQueryVariables>(GetPlaybackStatusDocument, variables),
    ...options
  }
    )};

useGetPlaybackStatusQuery.document = GetPlaybackStatusDocument;

useGetPlaybackStatusQuery.getKey = (variables?: GetPlaybackStatusQueryVariables) => variables === undefined ? ['GetPlaybackStatus'] : ['GetPlaybackStatus', variables];

export const CurrentlyPlayingSongDocument = new TypedDocumentString(`
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
    `);
export const PlaybackStatusDocument = new TypedDocumentString(`
    subscription PlaybackStatus {
  playbackStatus {
    status
  }
}
    `);
export const ResumePlaylistDocument = new TypedDocumentString(`
    mutation ResumePlaylist {
  playlistResume
}
    `);

export const useResumePlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ResumePlaylistMutation, TError, ResumePlaylistMutationVariables, TContext>) => {
    
    return useMutation<ResumePlaylistMutation, TError, ResumePlaylistMutationVariables, TContext>(
      {
    mutationKey: ['ResumePlaylist'],
    mutationFn: (variables?: ResumePlaylistMutationVariables) => fetchData<ResumePlaylistMutation, ResumePlaylistMutationVariables>(ResumePlaylistDocument, variables)(),
    ...options
  }
    )};

useResumePlaylistMutation.getKey = () => ['ResumePlaylist'];

export const ResumePlaylistTrackDocument = new TypedDocumentString(`
    mutation ResumePlaylistTrack {
  resumeTrack
}
    `);

export const useResumePlaylistTrackMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ResumePlaylistTrackMutation, TError, ResumePlaylistTrackMutationVariables, TContext>) => {
    
    return useMutation<ResumePlaylistTrackMutation, TError, ResumePlaylistTrackMutationVariables, TContext>(
      {
    mutationKey: ['ResumePlaylistTrack'],
    mutationFn: (variables?: ResumePlaylistTrackMutationVariables) => fetchData<ResumePlaylistTrackMutation, ResumePlaylistTrackMutationVariables>(ResumePlaylistTrackDocument, variables)(),
    ...options
  }
    )};

useResumePlaylistTrackMutation.getKey = () => ['ResumePlaylistTrack'];

export const PlaylistRemoveTrackDocument = new TypedDocumentString(`
    mutation PlaylistRemoveTrack($index: Int!) {
  playlistRemoveTrack(index: $index)
}
    `);

export const usePlaylistRemoveTrackMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlaylistRemoveTrackMutation, TError, PlaylistRemoveTrackMutationVariables, TContext>) => {
    
    return useMutation<PlaylistRemoveTrackMutation, TError, PlaylistRemoveTrackMutationVariables, TContext>(
      {
    mutationKey: ['PlaylistRemoveTrack'],
    mutationFn: (variables?: PlaylistRemoveTrackMutationVariables) => fetchData<PlaylistRemoveTrackMutation, PlaylistRemoveTrackMutationVariables>(PlaylistRemoveTrackDocument, variables)(),
    ...options
  }
    )};

usePlaylistRemoveTrackMutation.getKey = () => ['PlaylistRemoveTrack'];

export const StartPlaylistDocument = new TypedDocumentString(`
    mutation StartPlaylist($startIndex: Int, $elapsed: Int, $offset: Int) {
  playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset)
}
    `);

export const useStartPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<StartPlaylistMutation, TError, StartPlaylistMutationVariables, TContext>) => {
    
    return useMutation<StartPlaylistMutation, TError, StartPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['StartPlaylist'],
    mutationFn: (variables?: StartPlaylistMutationVariables) => fetchData<StartPlaylistMutation, StartPlaylistMutationVariables>(StartPlaylistDocument, variables)(),
    ...options
  }
    )};

useStartPlaylistMutation.getKey = () => ['StartPlaylist'];

export const InsertTracksDocument = new TypedDocumentString(`
    mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) {
  insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks)
}
    `);

export const useInsertTracksMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<InsertTracksMutation, TError, InsertTracksMutationVariables, TContext>) => {
    
    return useMutation<InsertTracksMutation, TError, InsertTracksMutationVariables, TContext>(
      {
    mutationKey: ['InsertTracks'],
    mutationFn: (variables?: InsertTracksMutationVariables) => fetchData<InsertTracksMutation, InsertTracksMutationVariables>(InsertTracksDocument, variables)(),
    ...options
  }
    )};

useInsertTracksMutation.getKey = () => ['InsertTracks'];

export const InsertDirectoryDocument = new TypedDocumentString(`
    mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) {
  insertDirectory(
    playlistId: $playlistId
    position: $position
    directory: $directory
  )
}
    `);

export const useInsertDirectoryMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<InsertDirectoryMutation, TError, InsertDirectoryMutationVariables, TContext>) => {
    
    return useMutation<InsertDirectoryMutation, TError, InsertDirectoryMutationVariables, TContext>(
      {
    mutationKey: ['InsertDirectory'],
    mutationFn: (variables?: InsertDirectoryMutationVariables) => fetchData<InsertDirectoryMutation, InsertDirectoryMutationVariables>(InsertDirectoryDocument, variables)(),
    ...options
  }
    )};

useInsertDirectoryMutation.getKey = () => ['InsertDirectory'];

export const InsertAlbumDocument = new TypedDocumentString(`
    mutation InsertAlbum($albumId: String!, $position: Int!) {
  insertAlbum(albumId: $albumId, position: $position)
}
    `);

export const useInsertAlbumMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<InsertAlbumMutation, TError, InsertAlbumMutationVariables, TContext>) => {
    
    return useMutation<InsertAlbumMutation, TError, InsertAlbumMutationVariables, TContext>(
      {
    mutationKey: ['InsertAlbum'],
    mutationFn: (variables?: InsertAlbumMutationVariables) => fetchData<InsertAlbumMutation, InsertAlbumMutationVariables>(InsertAlbumDocument, variables)(),
    ...options
  }
    )};

useInsertAlbumMutation.getKey = () => ['InsertAlbum'];

export const ShufflePlaylistDocument = new TypedDocumentString(`
    mutation ShufflePlaylist {
  shufflePlaylist
}
    `);

export const useShufflePlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ShufflePlaylistMutation, TError, ShufflePlaylistMutationVariables, TContext>) => {
    
    return useMutation<ShufflePlaylistMutation, TError, ShufflePlaylistMutationVariables, TContext>(
      {
    mutationKey: ['ShufflePlaylist'],
    mutationFn: (variables?: ShufflePlaylistMutationVariables) => fetchData<ShufflePlaylistMutation, ShufflePlaylistMutationVariables>(ShufflePlaylistDocument, variables)(),
    ...options
  }
    )};

useShufflePlaylistMutation.getKey = () => ['ShufflePlaylist'];

export const GetCurrentPlaylistDocument = new TypedDocumentString(`
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
    `);

export const useGetCurrentPlaylistQuery = <
      TData = GetCurrentPlaylistQuery,
      TError = unknown
    >(
      variables?: GetCurrentPlaylistQueryVariables,
      options?: Omit<UseQueryOptions<GetCurrentPlaylistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetCurrentPlaylistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetCurrentPlaylistQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetCurrentPlaylist'] : ['GetCurrentPlaylist', variables],
    queryFn: fetchData<GetCurrentPlaylistQuery, GetCurrentPlaylistQueryVariables>(GetCurrentPlaylistDocument, variables),
    ...options
  }
    )};

useGetCurrentPlaylistQuery.document = GetCurrentPlaylistDocument;

useGetCurrentPlaylistQuery.getKey = (variables?: GetCurrentPlaylistQueryVariables) => variables === undefined ? ['GetCurrentPlaylist'] : ['GetCurrentPlaylist', variables];

export const PlaylistChangedDocument = new TypedDocumentString(`
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
    `);
export const CreateSavedPlaylistDocument = new TypedDocumentString(`
    mutation CreateSavedPlaylist($name: String!, $description: String, $trackIds: [String!]) {
  createSavedPlaylist(name: $name, description: $description, trackIds: $trackIds) {
    id
    name
    description
    trackCount
  }
}
    `);

export const useCreateSavedPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<CreateSavedPlaylistMutation, TError, CreateSavedPlaylistMutationVariables, TContext>) => {
    
    return useMutation<CreateSavedPlaylistMutation, TError, CreateSavedPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['CreateSavedPlaylist'],
    mutationFn: (variables?: CreateSavedPlaylistMutationVariables) => fetchData<CreateSavedPlaylistMutation, CreateSavedPlaylistMutationVariables>(CreateSavedPlaylistDocument, variables)(),
    ...options
  }
    )};

useCreateSavedPlaylistMutation.getKey = () => ['CreateSavedPlaylist'];

export const UpdateSavedPlaylistDocument = new TypedDocumentString(`
    mutation UpdateSavedPlaylist($id: String!, $name: String!, $description: String) {
  updateSavedPlaylist(id: $id, name: $name, description: $description)
}
    `);

export const useUpdateSavedPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<UpdateSavedPlaylistMutation, TError, UpdateSavedPlaylistMutationVariables, TContext>) => {
    
    return useMutation<UpdateSavedPlaylistMutation, TError, UpdateSavedPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['UpdateSavedPlaylist'],
    mutationFn: (variables?: UpdateSavedPlaylistMutationVariables) => fetchData<UpdateSavedPlaylistMutation, UpdateSavedPlaylistMutationVariables>(UpdateSavedPlaylistDocument, variables)(),
    ...options
  }
    )};

useUpdateSavedPlaylistMutation.getKey = () => ['UpdateSavedPlaylist'];

export const DeleteSavedPlaylistDocument = new TypedDocumentString(`
    mutation DeleteSavedPlaylist($id: String!) {
  deleteSavedPlaylist(id: $id)
}
    `);

export const useDeleteSavedPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DeleteSavedPlaylistMutation, TError, DeleteSavedPlaylistMutationVariables, TContext>) => {
    
    return useMutation<DeleteSavedPlaylistMutation, TError, DeleteSavedPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['DeleteSavedPlaylist'],
    mutationFn: (variables?: DeleteSavedPlaylistMutationVariables) => fetchData<DeleteSavedPlaylistMutation, DeleteSavedPlaylistMutationVariables>(DeleteSavedPlaylistDocument, variables)(),
    ...options
  }
    )};

useDeleteSavedPlaylistMutation.getKey = () => ['DeleteSavedPlaylist'];

export const AddTracksToSavedPlaylistDocument = new TypedDocumentString(`
    mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) {
  addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds)
}
    `);

export const useAddTracksToSavedPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<AddTracksToSavedPlaylistMutation, TError, AddTracksToSavedPlaylistMutationVariables, TContext>) => {
    
    return useMutation<AddTracksToSavedPlaylistMutation, TError, AddTracksToSavedPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['AddTracksToSavedPlaylist'],
    mutationFn: (variables?: AddTracksToSavedPlaylistMutationVariables) => fetchData<AddTracksToSavedPlaylistMutation, AddTracksToSavedPlaylistMutationVariables>(AddTracksToSavedPlaylistDocument, variables)(),
    ...options
  }
    )};

useAddTracksToSavedPlaylistMutation.getKey = () => ['AddTracksToSavedPlaylist'];

export const RemoveTrackFromSavedPlaylistDocument = new TypedDocumentString(`
    mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) {
  removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId)
}
    `);

export const useRemoveTrackFromSavedPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<RemoveTrackFromSavedPlaylistMutation, TError, RemoveTrackFromSavedPlaylistMutationVariables, TContext>) => {
    
    return useMutation<RemoveTrackFromSavedPlaylistMutation, TError, RemoveTrackFromSavedPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['RemoveTrackFromSavedPlaylist'],
    mutationFn: (variables?: RemoveTrackFromSavedPlaylistMutationVariables) => fetchData<RemoveTrackFromSavedPlaylistMutation, RemoveTrackFromSavedPlaylistMutationVariables>(RemoveTrackFromSavedPlaylistDocument, variables)(),
    ...options
  }
    )};

useRemoveTrackFromSavedPlaylistMutation.getKey = () => ['RemoveTrackFromSavedPlaylist'];

export const PlaySavedPlaylistDocument = new TypedDocumentString(`
    mutation PlaySavedPlaylist($playlistId: String!) {
  playSavedPlaylist(playlistId: $playlistId)
}
    `);

export const usePlaySavedPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlaySavedPlaylistMutation, TError, PlaySavedPlaylistMutationVariables, TContext>) => {
    
    return useMutation<PlaySavedPlaylistMutation, TError, PlaySavedPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['PlaySavedPlaylist'],
    mutationFn: (variables?: PlaySavedPlaylistMutationVariables) => fetchData<PlaySavedPlaylistMutation, PlaySavedPlaylistMutationVariables>(PlaySavedPlaylistDocument, variables)(),
    ...options
  }
    )};

usePlaySavedPlaylistMutation.getKey = () => ['PlaySavedPlaylist'];

export const GetSavedPlaylistsDocument = new TypedDocumentString(`
    query GetSavedPlaylists {
  savedPlaylists {
    id
    name
    description
    image
    trackCount
    createdAt
    updatedAt
  }
}
    `);

export const useGetSavedPlaylistsQuery = <
      TData = GetSavedPlaylistsQuery,
      TError = unknown
    >(
      variables?: GetSavedPlaylistsQueryVariables,
      options?: Omit<UseQueryOptions<GetSavedPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSavedPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSavedPlaylistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetSavedPlaylists'] : ['GetSavedPlaylists', variables],
    queryFn: fetchData<GetSavedPlaylistsQuery, GetSavedPlaylistsQueryVariables>(GetSavedPlaylistsDocument, variables),
    ...options
  }
    )};

useGetSavedPlaylistsQuery.document = GetSavedPlaylistsDocument;

useGetSavedPlaylistsQuery.getKey = (variables?: GetSavedPlaylistsQueryVariables) => variables === undefined ? ['GetSavedPlaylists'] : ['GetSavedPlaylists', variables];

export const GetSavedPlaylistDocument = new TypedDocumentString(`
    query GetSavedPlaylist($id: String!) {
  savedPlaylist(id: $id) {
    id
    name
    description
    image
    trackCount
    createdAt
    updatedAt
  }
}
    `);

export const useGetSavedPlaylistQuery = <
      TData = GetSavedPlaylistQuery,
      TError = unknown
    >(
      variables: GetSavedPlaylistQueryVariables,
      options?: Omit<UseQueryOptions<GetSavedPlaylistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSavedPlaylistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSavedPlaylistQuery, TError, TData>(
      {
    queryKey: ['GetSavedPlaylist', variables],
    queryFn: fetchData<GetSavedPlaylistQuery, GetSavedPlaylistQueryVariables>(GetSavedPlaylistDocument, variables),
    ...options
  }
    )};

useGetSavedPlaylistQuery.document = GetSavedPlaylistDocument;

useGetSavedPlaylistQuery.getKey = (variables: GetSavedPlaylistQueryVariables) => ['GetSavedPlaylist', variables];

export const GetSavedPlaylistTracksDocument = new TypedDocumentString(`
    query GetSavedPlaylistTracks($playlistId: String!) {
  savedPlaylistTracks(playlistId: $playlistId) {
    id
    title
    artist
    album
    albumArt
    artistId
    albumId
    path
    length
    tracknum
  }
}
    `);

export const useGetSavedPlaylistTracksQuery = <
      TData = GetSavedPlaylistTracksQuery,
      TError = unknown
    >(
      variables: GetSavedPlaylistTracksQueryVariables,
      options?: Omit<UseQueryOptions<GetSavedPlaylistTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSavedPlaylistTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSavedPlaylistTracksQuery, TError, TData>(
      {
    queryKey: ['GetSavedPlaylistTracks', variables],
    queryFn: fetchData<GetSavedPlaylistTracksQuery, GetSavedPlaylistTracksQueryVariables>(GetSavedPlaylistTracksDocument, variables),
    ...options
  }
    )};

useGetSavedPlaylistTracksQuery.document = GetSavedPlaylistTracksDocument;

useGetSavedPlaylistTracksQuery.getKey = (variables: GetSavedPlaylistTracksQueryVariables) => ['GetSavedPlaylistTracks', variables];

export const SaveSettingsDocument = new TypedDocumentString(`
    mutation SaveSettings($settings: NewGlobalSettings!) {
  saveSettings(settings: $settings)
}
    `);

export const useSaveSettingsMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<SaveSettingsMutation, TError, SaveSettingsMutationVariables, TContext>) => {
    
    return useMutation<SaveSettingsMutation, TError, SaveSettingsMutationVariables, TContext>(
      {
    mutationKey: ['SaveSettings'],
    mutationFn: (variables?: SaveSettingsMutationVariables) => fetchData<SaveSettingsMutation, SaveSettingsMutationVariables>(SaveSettingsDocument, variables)(),
    ...options
  }
    )};

useSaveSettingsMutation.getKey = () => ['SaveSettings'];

export const GetGlobalSettingsDocument = new TypedDocumentString(`
    query GetGlobalSettings {
  globalSettings {
    musicDir
    volume
    playlistShuffle
    repeatMode
    bass
    bassCutoff
    treble
    trebleCutoff
    crossfade
    fadeOnStop
    crossfadeFadeInDelay
    crossfadeFadeInDuration
    crossfadeFadeOutDelay
    crossfadeFadeOutDuration
    crossfadeFadeOutMixmode
    balance
    stereoWidth
    stereoswMode
    surroundEnabled
    surroundBalance
    surroundFx1
    surroundFx2
    partyMode
    ditheringEnabled
    channelConfig
    playerName
    eqEnabled
    eqBandSettings {
      q
      cutoff
      gain
    }
    replaygainSettings {
      noclip
      type
      preamp
    }
  }
}
    `);

export const useGetGlobalSettingsQuery = <
      TData = GetGlobalSettingsQuery,
      TError = unknown
    >(
      variables?: GetGlobalSettingsQueryVariables,
      options?: Omit<UseQueryOptions<GetGlobalSettingsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetGlobalSettingsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetGlobalSettingsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetGlobalSettings'] : ['GetGlobalSettings', variables],
    queryFn: fetchData<GetGlobalSettingsQuery, GetGlobalSettingsQueryVariables>(GetGlobalSettingsDocument, variables),
    ...options
  }
    )};

useGetGlobalSettingsQuery.document = GetGlobalSettingsDocument;

useGetGlobalSettingsQuery.getKey = (variables?: GetGlobalSettingsQueryVariables) => variables === undefined ? ['GetGlobalSettings'] : ['GetGlobalSettings', variables];

export const PlaySmartPlaylistDocument = new TypedDocumentString(`
    mutation PlaySmartPlaylist($id: String!) {
  playSmartPlaylist(id: $id)
}
    `);

export const usePlaySmartPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlaySmartPlaylistMutation, TError, PlaySmartPlaylistMutationVariables, TContext>) => {
    
    return useMutation<PlaySmartPlaylistMutation, TError, PlaySmartPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['PlaySmartPlaylist'],
    mutationFn: (variables?: PlaySmartPlaylistMutationVariables) => fetchData<PlaySmartPlaylistMutation, PlaySmartPlaylistMutationVariables>(PlaySmartPlaylistDocument, variables)(),
    ...options
  }
    )};

usePlaySmartPlaylistMutation.getKey = () => ['PlaySmartPlaylist'];

export const GetSmartPlaylistsDocument = new TypedDocumentString(`
    query GetSmartPlaylists {
  smartPlaylists {
    id
    name
    description
    image
    isSystem
    createdAt
    updatedAt
  }
}
    `);

export const useGetSmartPlaylistsQuery = <
      TData = GetSmartPlaylistsQuery,
      TError = unknown
    >(
      variables?: GetSmartPlaylistsQueryVariables,
      options?: Omit<UseQueryOptions<GetSmartPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSmartPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSmartPlaylistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetSmartPlaylists'] : ['GetSmartPlaylists', variables],
    queryFn: fetchData<GetSmartPlaylistsQuery, GetSmartPlaylistsQueryVariables>(GetSmartPlaylistsDocument, variables),
    ...options
  }
    )};

useGetSmartPlaylistsQuery.document = GetSmartPlaylistsDocument;

useGetSmartPlaylistsQuery.getKey = (variables?: GetSmartPlaylistsQueryVariables) => variables === undefined ? ['GetSmartPlaylists'] : ['GetSmartPlaylists', variables];

export const GetSmartPlaylistDocument = new TypedDocumentString(`
    query GetSmartPlaylist($id: String!) {
  smartPlaylist(id: $id) {
    id
    name
    description
    image
    isSystem
    createdAt
    updatedAt
  }
}
    `);

export const useGetSmartPlaylistQuery = <
      TData = GetSmartPlaylistQuery,
      TError = unknown
    >(
      variables: GetSmartPlaylistQueryVariables,
      options?: Omit<UseQueryOptions<GetSmartPlaylistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSmartPlaylistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSmartPlaylistQuery, TError, TData>(
      {
    queryKey: ['GetSmartPlaylist', variables],
    queryFn: fetchData<GetSmartPlaylistQuery, GetSmartPlaylistQueryVariables>(GetSmartPlaylistDocument, variables),
    ...options
  }
    )};

useGetSmartPlaylistQuery.document = GetSmartPlaylistDocument;

useGetSmartPlaylistQuery.getKey = (variables: GetSmartPlaylistQueryVariables) => ['GetSmartPlaylist', variables];

export const GetSmartPlaylistTracksDocument = new TypedDocumentString(`
    query GetSmartPlaylistTracks($id: String!) {
  smartPlaylistTracks(id: $id) {
    id
    title
    artist
    album
    albumArt
    artistId
    albumId
    path
    length
    tracknum
  }
}
    `);

export const useGetSmartPlaylistTracksQuery = <
      TData = GetSmartPlaylistTracksQuery,
      TError = unknown
    >(
      variables: GetSmartPlaylistTracksQueryVariables,
      options?: Omit<UseQueryOptions<GetSmartPlaylistTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSmartPlaylistTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSmartPlaylistTracksQuery, TError, TData>(
      {
    queryKey: ['GetSmartPlaylistTracks', variables],
    queryFn: fetchData<GetSmartPlaylistTracksQuery, GetSmartPlaylistTracksQueryVariables>(GetSmartPlaylistTracksDocument, variables),
    ...options
  }
    )};

useGetSmartPlaylistTracksQuery.document = GetSmartPlaylistTracksDocument;

useGetSmartPlaylistTracksQuery.getKey = (variables: GetSmartPlaylistTracksQueryVariables) => ['GetSmartPlaylistTracks', variables];

export const AdjustVolumeDocument = new TypedDocumentString(`
    mutation AdjustVolume($steps: Int!) {
  adjustVolume(steps: $steps)
}
    `);

export const useAdjustVolumeMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<AdjustVolumeMutation, TError, AdjustVolumeMutationVariables, TContext>) => {
    
    return useMutation<AdjustVolumeMutation, TError, AdjustVolumeMutationVariables, TContext>(
      {
    mutationKey: ['AdjustVolume'],
    mutationFn: (variables?: AdjustVolumeMutationVariables) => fetchData<AdjustVolumeMutation, AdjustVolumeMutationVariables>(AdjustVolumeDocument, variables)(),
    ...options
  }
    )};

useAdjustVolumeMutation.getKey = () => ['AdjustVolume'];

export const GetRockboxVersionDocument = new TypedDocumentString(`
    query GetRockboxVersion {
  rockboxVersion
}
    `);

export const useGetRockboxVersionQuery = <
      TData = GetRockboxVersionQuery,
      TError = unknown
    >(
      variables?: GetRockboxVersionQueryVariables,
      options?: Omit<UseQueryOptions<GetRockboxVersionQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetRockboxVersionQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetRockboxVersionQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetRockboxVersion'] : ['GetRockboxVersion', variables],
    queryFn: fetchData<GetRockboxVersionQuery, GetRockboxVersionQueryVariables>(GetRockboxVersionDocument, variables),
    ...options
  }
    )};

useGetRockboxVersionQuery.document = GetRockboxVersionDocument;

useGetRockboxVersionQuery.getKey = (variables?: GetRockboxVersionQueryVariables) => variables === undefined ? ['GetRockboxVersion'] : ['GetRockboxVersion', variables];

export const GetGlobalStatusDocument = new TypedDocumentString(`
    query GetGlobalStatus {
  globalStatus {
    resumeIndex
    resumeCrc32
    resumeOffset
    resumeElapsed
  }
}
    `);

export const useGetGlobalStatusQuery = <
      TData = GetGlobalStatusQuery,
      TError = unknown
    >(
      variables?: GetGlobalStatusQueryVariables,
      options?: Omit<UseQueryOptions<GetGlobalStatusQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetGlobalStatusQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetGlobalStatusQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetGlobalStatus'] : ['GetGlobalStatus', variables],
    queryFn: fetchData<GetGlobalStatusQuery, GetGlobalStatusQueryVariables>(GetGlobalStatusDocument, variables),
    ...options
  }
    )};

useGetGlobalStatusQuery.document = GetGlobalStatusDocument;

useGetGlobalStatusQuery.getKey = (variables?: GetGlobalStatusQueryVariables) => variables === undefined ? ['GetGlobalStatus'] : ['GetGlobalStatus', variables];

export const useCurrentlyPlayingSongSubscription = () =>
  useSubscription<CurrentlyPlayingSongSubscription>(
    CurrentlyPlayingSongDocument.toString()
  );

export const usePlaybackStatusSubscription = () =>
  useSubscription<PlaybackStatusSubscription>(
    PlaybackStatusDocument.toString()
  );

export type GetVolumeQuery = { __typename?: 'Query', volume: { __typename?: 'VolumeInfo', volume: number, min: number, max: number } };

export const GetVolumeDocument = new TypedDocumentString(`
    query GetVolume {
  volume {
    volume
    min
    max
  }
}
    `);

export const useGetVolumeQuery = <
      TData = GetVolumeQuery,
      TError = unknown
    >(
      variables?: Record<string, never>,
      options?: Omit<UseQueryOptions<GetVolumeQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetVolumeQuery, TError, TData>['queryKey'] }
    ) => {
    return useQuery<GetVolumeQuery, TError, TData>(
      {
    queryKey: ['GetVolume'],
    queryFn: fetchData<GetVolumeQuery, Record<string, never>>(GetVolumeDocument, variables),
    ...options
  }
    )};

useGetVolumeQuery.document = GetVolumeDocument;
useGetVolumeQuery.getKey = () => ['GetVolume'];

export const usePlaylistChangedSubscription = () =>
  useSubscription<PlaylistChangedSubscription>(
    PlaylistChangedDocument.toString()
  );
