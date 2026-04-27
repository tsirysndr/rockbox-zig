export { RockboxClient } from './client.js';
export type { RockboxClientConfig } from './client.js';

export type { RockboxEventMap } from './events.js';
export { TypedEventEmitter } from './events.js';

export type { RockboxPlugin, PluginContext } from './plugin.js';

export { RockboxError, RockboxNetworkError, RockboxGraphQLError } from './errors.js';

export {
  PlaybackStatus,
  RepeatMode,
  ChannelConfig,
  ReplaygainType,
  InsertPosition,
  isDirectory,
} from './types.js';

export type {
  Track,
  Album,
  Artist,
  SearchResults,
  Playlist,
  SavedPlaylist,
  SavedPlaylistFolder,
  SmartPlaylist,
  TrackStats,
  Device,
  Entry,
  SystemStatus,
  UserSettings,
  PartialUserSettings,
  EqBandSetting,
  ReplaygainSettings,
  CompressorSettings,
} from './types.js';

export type { CreatePlaylistInput, UpdatePlaylistInput } from './api/saved-playlists.js';
export type { CreateSmartPlaylistInput, UpdateSmartPlaylistInput } from './api/smart-playlists.js';
