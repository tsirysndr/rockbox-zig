import { requireNativeModule } from "expo";
import type { EventSubscription } from "expo-modules-core";

/**
 * Track snapshot returned by `currentTrack()`. Mirrors the JSON shape
 * produced by `rb_current_track_json` in `crates/expo/src/lib.rs`.
 */
export type TrackSnapshot = {
  id: string;
  path: string;
  title: string;
  artist: string;
  album: string;
  album_art?: string | null;
  duration_ms: number;
  elapsed_ms: number;
};

/**
 * Server playback status. Numeric codes mirror the rockbox internal status
 * enum: 0 = stopped, 1 = playing, 2 = paused.
 */
export type StatusSnapshot = {
  status: 0 | 1 | 2;
};

/** Streamed playlist snapshot (queue position + tracks). */
export type PlaylistSnapshot = {
  index: number;
  amount: number;
  tracks: TrackSnapshot[];
};

export type StreamErrorEvent = {
  subId: number;
  stream:
    | "rockbox.status"
    | "rockbox.currentTrack"
    | "rockbox.playlist"
    | "rockbox.library"
    | "rockbox.discovery";
  error: string;
};

/**
 * One resolved service from the mDNS discovery stream. Mirrors the JSON shape
 * produced by `rb_subscribe_discovery` in `crates/expo`.
 */
export type DiscoveredService = {
  name: string;
  fullname: string;
  hostname: string;
  port: number;
  addresses: string[];
  properties: Record<string, string>;
};

/** Event payload map. Matches `Events(...)` declared in the native modules. */
export type RockboxRpcEvents = {
  "rockbox.status": (s: StatusSnapshot) => void;
  "rockbox.currentTrack": (t: TrackSnapshot) => void;
  "rockbox.playlist": (p: PlaylistSnapshot) => void;
  "rockbox.library": (l: unknown) => void;
  "rockbox.discovery": (s: DiscoveredService) => void;
  "rockbox.error": (e: StreamErrorEvent) => void;
};

// Loose JSON shapes — proto messages are returned as-is. These are unknowns
// because the proto types aren't generated on the JS side; consumers can cast
// or define their own row types as needed.
export type Json = unknown;

type RockboxRpcNative = {
  /** Configure the gRPC server URL. Call once at app startup. */
  setServerUrl(url: string): void;
  /** Configure the rockboxd HTTP base URL (used by `getDevices` etc.). */
  setHttpUrl(url: string): void;
  /** Round-trip Status RPC; resolves with `true` if the server replied. */
  ping(): Promise<boolean>;

  // Playback
  play(): Promise<void>;
  pause(): Promise<void>;
  playPause(): Promise<void>;
  next(): Promise<void>;
  prev(): Promise<void>;
  seek(positionMs: number): Promise<void>;

  // Read
  status(): Promise<StatusSnapshot>;
  currentTrack(): Promise<TrackSnapshot>;

  // Library
  likeTrack(id: string): Promise<void>;
  unlikeTrack(id: string): Promise<void>;

  // Playback (extended)
  resumeTrack(): Promise<void>;
  playlistResume(): Promise<void>;
  playAllTracks(): Promise<void>;
  playTrack(path: string): Promise<void>;
  playAlbum(albumId: string, shuffle: boolean): Promise<void>;
  playArtistTracks(artistId: string, shuffle: boolean): Promise<void>;
  playDirectory(
    path: string,
    shuffle: boolean,
    /** -1 to omit; otherwise zero-based start position */
    position: number,
  ): Promise<void>;

  // Queue
  jumpToQueuePosition(pos: number): Promise<void>;
  shufflePlaylist(): Promise<void>;
  insertTracks(paths: string[], position: number, shuffle: boolean): Promise<void>;
  insertTrackNext(path: string): Promise<void>;
  insertTrackLast(path: string): Promise<void>;
  insertDirectory(path: string, position: number): Promise<void>;
  removeFromQueue(pos: number): Promise<void>;
  getPlaylistCurrent(): Promise<Json>;

  // Library
  getTracks(): Promise<Json>;
  getArtists(): Promise<Json>;
  getAlbums(): Promise<Json>;
  getLikedAlbums(): Promise<Json>;
  getArtist(id: string): Promise<Json>;
  getAlbum(id: string): Promise<Json>;
  getLikedTracks(): Promise<Json>;
  search(term: string): Promise<Json>;

  // Sound
  adjustVolume(steps: number): Promise<void>;
  soundCurrent(setting: number): Promise<Json>;

  // Settings
  saveShuffle(enabled: boolean): Promise<void>;
  saveRepeat(mode: number): Promise<void>;
  getGlobalSettings(): Promise<Json>;
  getGlobalStatus(): Promise<Json>;

  // Browse
  treeGetEntries(path: string | null): Promise<Json>;

  // Saved playlists
  getSavedPlaylists(): Promise<Json>;
  createSavedPlaylist(
    name: string,
    description: string | null,
    trackIds: string[],
  ): Promise<void>;
  updateSavedPlaylist(
    id: string,
    name: string,
    description: string | null,
  ): Promise<void>;
  deleteSavedPlaylist(id: string): Promise<void>;
  addTrackToPlaylist(playlistId: string, trackId: string): Promise<void>;
  removeTrackFromPlaylist(playlistId: string, trackId: string): Promise<void>;
  getSavedPlaylistTracks(playlistId: string): Promise<Json>;
  playSavedPlaylist(playlistId: string): Promise<void>;

  // Genres
  getGenres(): Promise<Json>;
  getGenre(id: string): Promise<Json>;
  getGenreTracks(id: string): Promise<Json>;
  getGenreAlbums(id: string): Promise<Json>;
  getGenreArtists(id: string): Promise<Json>;

  // Smart playlists
  getSmartPlaylists(): Promise<Json>;
  getSmartPlaylistTracks(id: string): Promise<Json>;
  playSmartPlaylist(id: string): Promise<void>;

  // Bluetooth
  bluetoothAvailable(): Promise<boolean>;
  scanBluetooth(): Promise<void>;
  getBluetoothDevices(): Promise<Json>;
  connectBluetooth(address: string): Promise<void>;
  disconnectBluetooth(address: string): Promise<void>;

  // Cast / AirPlay output devices (HTTP REST under the hood).
  getDevices(): Promise<Json>;
  connectDevice(id: string): Promise<void>;
  disconnectDevice(id: string): Promise<void>;

  // Streaming subscriptions — return an opaque numeric subscription id.
  // Pair with `unsubscribe(id)` to tear down. Events fire on the registered
  // listener channels (see `RockboxRpcEvents`).
  subscribeStatus(): number;
  subscribeCurrentTrack(): number;
  subscribePlaylist(): number;
  subscribeLibrary(): number;
  subscribeDiscovery(serviceName: string): number;
  unsubscribe(subId: number): void;

  // mDNS / Bonjour service-name constants exported by `rockbox-discovery`.
  rockboxServiceName(): string;
  chromecastServiceName(): string;

  /**
   * Android-only: returns true if the app holds MANAGE_EXTERNAL_STORAGE
   * ("All files access"), required to scan /storage/emulated/0/Music on
   * API 33+. Always true on iOS / pre-Android-11.
   */
  hasAllFilesAccess(): boolean;
  /**
   * Android-only: opens system Settings → "All files access" for this
   * package. Returns true if the intent launched. iOS no-ops to false.
   * The grant is one-time; check hasAllFilesAccess() after the user returns.
   */
  requestAllFilesAccess(): boolean;

  /**
   * Force a full library rescan of $ROCKBOX_LIBRARY (the music dir passed
   * at daemon boot). Returns 0 if queued, -1 if the daemon isn't running,
   * -38 in remote-only builds (no embedded daemon). The scan runs on a
   * background thread — listen on the gRPC `ScanCompleted` event or just
   * tail logcat ("scan: done, N files") to know when it finishes.
   */
  rescanLibrary(): number;

  // Event API (provided by Expo Modules' EventEmitter base).
  addListener<K extends keyof RockboxRpcEvents>(
    event: K,
    listener: RockboxRpcEvents[K],
  ): EventSubscription;
  removeAllListeners(event: keyof RockboxRpcEvents): void;
};

const RockboxRpc = requireNativeModule<RockboxRpcNative>("RockboxRpc");

export default RockboxRpc;
