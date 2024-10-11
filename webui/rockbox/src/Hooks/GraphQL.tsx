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
  bio?: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  image?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
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
  adjustVolume: Scalars['String']['output'];
  beepPlay: Scalars['String']['output'];
  fastForwardRewind: Scalars['String']['output'];
  flushAndReloadTracks: Scalars['String']['output'];
  hardStop: Scalars['String']['output'];
  insertPlaylist: Scalars['String']['output'];
  keyclickClick: Scalars['String']['output'];
  next: Scalars['String']['output'];
  pause: Scalars['String']['output'];
  pcmbufFade: Scalars['String']['output'];
  pcmbufPlay: Scalars['String']['output'];
  pcmbufSetLowLatency: Scalars['String']['output'];
  play: Scalars['String']['output'];
  playlistCreate: Scalars['Int']['output'];
  playlistInsertDirectory: Scalars['Int']['output'];
  playlistInsertTracks: Scalars['Int']['output'];
  playlistRemoveAllTracks: Scalars['Int']['output'];
  playlistResume: Scalars['String']['output'];
  playlistSetModified: Scalars['String']['output'];
  playlistStart: Scalars['Int']['output'];
  playlistSync: Scalars['String']['output'];
  previous: Scalars['String']['output'];
  resume: Scalars['String']['output'];
  resumeTrack: Scalars['String']['output'];
  setPitch: Scalars['String']['output'];
  shufflePlaylist: Scalars['Int']['output'];
  soundMax: Scalars['String']['output'];
  soundMin: Scalars['String']['output'];
  soundSet: Scalars['String']['output'];
  soundUnit: Scalars['String']['output'];
  systemSoundPlay: Scalars['String']['output'];
};


export type MutationFastForwardRewindArgs = {
  newTime: Scalars['Int']['input'];
};


export type MutationInsertPlaylistArgs = {
  playlistId: Scalars['String']['input'];
  position: Scalars['Int']['input'];
};


export type MutationPlayArgs = {
  elapsed: Scalars['Int']['input'];
  offset: Scalars['Int']['input'];
};


export type MutationPlaylistCreateArgs = {
  name: Scalars['String']['input'];
  tracks: Array<Scalars['String']['input']>;
};


export type MutationPlaylistInsertDirectoryArgs = {
  directory: Scalars['String']['input'];
  position: Scalars['Int']['input'];
};


export type MutationPlaylistInsertTracksArgs = {
  position: Scalars['Int']['input'];
  tracks: Array<Scalars['String']['input']>;
};

export type Playlist = {
  __typename?: 'Playlist';
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
  path: Scalars['String']['input'];
};

export type ReplaygainSettings = {
  __typename?: 'ReplaygainSettings';
  noclip: Scalars['Boolean']['output'];
  preamp: Scalars['Int']['output'];
  type: Scalars['Int']['output'];
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
  path: Scalars['String']['input'];
}>;


export type GetEntriesQuery = { __typename?: 'Query', treeGetEntries: Array<{ __typename?: 'Entry', name: string, timeWrite: number }> };

export type GetAlbumsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAlbumsQuery = { __typename?: 'Query', albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, artist: string, album: string, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> }> };

export type GetArtistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetArtistsQuery = { __typename?: 'Query', artists: Array<{ __typename?: 'Artist', id: string, name: string }> };

export type TracksQueryVariables = Exact<{ [key: string]: never; }>;


export type TracksQuery = { __typename?: 'Query', tracks: Array<{ __typename?: 'Track', id?: string | null, tracknum: number, title: string, artist: string, album: string, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> };

export type GetAlbumQueryVariables = Exact<{
  id: Scalars['String']['input'];
}>;


export type GetAlbumQuery = { __typename?: 'Query', album?: { __typename?: 'Album', id: string, title: string, artist: string, albumArt?: string | null, year: number, yearString: string, artistId: string, md5: string, tracks: Array<{ __typename?: 'Track', id?: string | null, title: string, tracknum: number, artist: string, album: string, albumArtist: string, artistId?: string | null, albumId?: string | null, path: string, length: number }> } | null };

export type GetRockboxVersionQueryVariables = Exact<{ [key: string]: never; }>;


export type GetRockboxVersionQuery = { __typename?: 'Query', rockboxVersion: string };


export const GetEntriesDocument = gql`
    query GetEntries($path: String!) {
  treeGetEntries(path: $path) {
    name
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
export function useGetEntriesQuery(baseOptions: Apollo.QueryHookOptions<GetEntriesQuery, GetEntriesQueryVariables> & ({ variables: GetEntriesQueryVariables; skip?: boolean; } | { skip: boolean; }) ) {
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
export const TracksDocument = gql`
    query Tracks {
  tracks {
    id
    tracknum
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