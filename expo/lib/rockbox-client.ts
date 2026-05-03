/**
 * Thin wrapper around the `rockbox-rpc` native module.
 *
 * - On iOS / Android, delegates to the native module (which calls into the
 *   `rockbox-expo` Rust crate via JNI / Swift's `@_silgen_name`).
 * - On web / when the native module isn't built yet, throws a friendly error
 *   from `requireNativeModule`. Callers should guard with `isAvailable` and
 *   fall back to the mock `PlayerProvider` ticker.
 */
import { Platform } from "react-native";
import type {
  DiscoveredService,
  PlaylistSnapshot,
  StatusSnapshot,
  StreamErrorEvent,
  TrackSnapshot,
} from "rockbox-rpc";

let nativeModule:
  | typeof import("rockbox-rpc").default
  | null = null;

if (Platform.OS === "ios" || Platform.OS === "android") {
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    nativeModule = require("rockbox-rpc").default;
  } catch {
    nativeModule = null;
  }
}

export const isAvailable = nativeModule !== null;

function require_(): NonNullable<typeof nativeModule> {
  if (!nativeModule) {
    throw new Error(
      "rockbox-rpc native module not available — run `bun run build:ios` or `build:android` from `expo/modules/rockbox-rpc/`",
    );
  }
  return nativeModule;
}

export const RockboxClient = {
  isAvailable,

  setServerUrl(url: string) {
    require_().setServerUrl(url);
  },

  ping(): Promise<boolean> {
    return require_().ping();
  },

  // Playback
  play() {
    return require_().play();
  },
  pause() {
    return require_().pause();
  },
  playPause() {
    return require_().playPause();
  },
  next() {
    return require_().next();
  },
  prev() {
    return require_().prev();
  },
  seek(positionMs: number) {
    return require_().seek(positionMs);
  },

  // Read
  status(): Promise<StatusSnapshot> {
    return require_().status();
  },
  currentTrack(): Promise<TrackSnapshot> {
    return require_().currentTrack();
  },

  // Playback (extended)
  resumeTrack() {
    return require_().resumeTrack();
  },
  playlistResume() {
    return require_().playlistResume();
  },
  playAllTracks() {
    return require_().playAllTracks();
  },
  playTrack(path: string) {
    return require_().playTrack(path);
  },
  playAlbum(albumId: string, shuffle = false) {
    return require_().playAlbum(albumId, shuffle);
  },
  playArtistTracks(artistId: string, shuffle = false) {
    return require_().playArtistTracks(artistId, shuffle);
  },
  playDirectory(path: string, shuffle = false, position = -1) {
    return require_().playDirectory(path, shuffle, position);
  },

  // Queue
  jumpToQueuePosition(pos: number) {
    return require_().jumpToQueuePosition(pos);
  },
  shufflePlaylistAtStart() {
    return require_().shufflePlaylist();
  },
  insertTracks(paths: string[], position: number, shuffle = false) {
    return require_().insertTracks(paths, position, shuffle);
  },
  insertTrackNext(path: string) {
    return require_().insertTrackNext(path);
  },
  insertTrackLast(path: string) {
    return require_().insertTrackLast(path);
  },
  insertDirectory(path: string, position: number) {
    return require_().insertDirectory(path, position);
  },
  removeFromQueue(pos: number) {
    return require_().removeFromQueue(pos);
  },
  getPlaylistCurrent() {
    return require_().getPlaylistCurrent();
  },

  // Library
  likeTrack(id: string) {
    return require_().likeTrack(id);
  },
  unlikeTrack(id: string) {
    return require_().unlikeTrack(id);
  },
  getTracks() {
    return require_().getTracks();
  },
  getArtists() {
    return require_().getArtists();
  },
  getAlbum(id: string) {
    return require_().getAlbum(id);
  },
  getLikedTracks() {
    return require_().getLikedTracks();
  },
  search(term: string) {
    return require_().search(term);
  },

  // Sound
  adjustVolume(steps: number) {
    return require_().adjustVolume(steps);
  },
  soundCurrent(setting: number) {
    return require_().soundCurrent(setting);
  },

  // Settings
  saveShuffle(enabled: boolean) {
    return require_().saveShuffle(enabled);
  },
  saveRepeat(mode: number) {
    return require_().saveRepeat(mode);
  },
  getGlobalSettings() {
    return require_().getGlobalSettings();
  },
  getGlobalStatus() {
    return require_().getGlobalStatus();
  },

  // Browse
  treeGetEntries(path: string | null = null) {
    return require_().treeGetEntries(path);
  },

  // Saved playlists
  getSavedPlaylists() {
    return require_().getSavedPlaylists();
  },
  createSavedPlaylist(
    name: string,
    description: string | null = null,
    trackIds: string[] = [],
  ) {
    return require_().createSavedPlaylist(name, description, trackIds);
  },
  updateSavedPlaylist(id: string, name: string, description: string | null = null) {
    return require_().updateSavedPlaylist(id, name, description);
  },
  deleteSavedPlaylist(id: string) {
    return require_().deleteSavedPlaylist(id);
  },
  addTrackToPlaylist(playlistId: string, trackId: string) {
    return require_().addTrackToPlaylist(playlistId, trackId);
  },
  removeTrackFromPlaylist(playlistId: string, trackId: string) {
    return require_().removeTrackFromPlaylist(playlistId, trackId);
  },
  getSavedPlaylistTracks(playlistId: string) {
    return require_().getSavedPlaylistTracks(playlistId);
  },
  playSavedPlaylist(playlistId: string) {
    return require_().playSavedPlaylist(playlistId);
  },

  // Smart playlists
  getSmartPlaylists() {
    return require_().getSmartPlaylists();
  },
  getSmartPlaylistTracks(id: string) {
    return require_().getSmartPlaylistTracks(id);
  },
  playSmartPlaylist(id: string) {
    return require_().playSmartPlaylist(id);
  },

  // Bluetooth
  bluetoothAvailable() {
    return require_().bluetoothAvailable();
  },
  getBluetoothDevices() {
    return require_().getBluetoothDevices();
  },
  connectBluetooth(address: string) {
    return require_().connectBluetooth(address);
  },
  disconnectBluetooth(address: string) {
    return require_().disconnectBluetooth(address);
  },

  // mDNS service-name constants
  rockboxServiceName() {
    return require_().rockboxServiceName();
  },
  chromecastServiceName() {
    return require_().chromecastServiceName();
  },

  // ── Streaming subscriptions ─────────────────────────────────────────────
  // Each helper returns an unsubscribe function that tears down both the
  // event listener and the native subscription.

  subscribeStatus(
    onEvent: (s: StatusSnapshot) => void,
    onError?: (e: StreamErrorEvent) => void,
  ): () => void {
    const m = require_();
    const subId = m.subscribeStatus();
    const evtSub = m.addListener("rockbox.status", onEvent);
    const errSub = onError
      ? m.addListener("rockbox.error", (e) => {
          if (e.subId === subId) onError(e);
        })
      : null;
    return () => {
      evtSub.remove();
      errSub?.remove();
      m.unsubscribe(subId);
    };
  },

  subscribeCurrentTrack(
    onEvent: (t: TrackSnapshot) => void,
    onError?: (e: StreamErrorEvent) => void,
  ): () => void {
    const m = require_();
    const subId = m.subscribeCurrentTrack();
    const evtSub = m.addListener("rockbox.currentTrack", onEvent);
    const errSub = onError
      ? m.addListener("rockbox.error", (e) => {
          if (e.subId === subId) onError(e);
        })
      : null;
    return () => {
      evtSub.remove();
      errSub?.remove();
      m.unsubscribe(subId);
    };
  },

  subscribePlaylist(
    onEvent: (p: PlaylistSnapshot) => void,
    onError?: (e: StreamErrorEvent) => void,
  ): () => void {
    const m = require_();
    const subId = m.subscribePlaylist();
    const evtSub = m.addListener("rockbox.playlist", onEvent);
    const errSub = onError
      ? m.addListener("rockbox.error", (e) => {
          if (e.subId === subId) onError(e);
        })
      : null;
    return () => {
      evtSub.remove();
      errSub?.remove();
      m.unsubscribe(subId);
    };
  },

  subscribeLibrary(
    onEvent: (snapshot: unknown) => void,
    onError?: (e: StreamErrorEvent) => void,
  ): () => void {
    const m = require_();
    const subId = m.subscribeLibrary();
    const evtSub = m.addListener("rockbox.library", onEvent);
    const errSub = onError
      ? m.addListener("rockbox.error", (e) => {
          if (e.subId === subId) onError(e);
        })
      : null;
    return () => {
      evtSub.remove();
      errSub?.remove();
      m.unsubscribe(subId);
    };
  },

  /**
   * Discover rockbox / Chromecast / arbitrary mDNS services on the LAN.
   * Defaults to `_rockbox._tcp.local.` when `serviceName` is omitted.
   */
  subscribeDiscovery(
    onEvent: (service: DiscoveredService) => void,
    onError?: (e: StreamErrorEvent) => void,
    serviceName?: string,
  ): () => void {
    const m = require_();
    const name = serviceName ?? m.rockboxServiceName();
    const subId = m.subscribeDiscovery(name);
    const evtSub = m.addListener("rockbox.discovery", onEvent);
    const errSub = onError
      ? m.addListener("rockbox.error", (e) => {
          if (e.subId === subId) onError(e);
        })
      : null;
    return () => {
      evtSub.remove();
      errSub?.remove();
      m.unsubscribe(subId);
    };
  },
};

export type {
  DiscoveredService,
  PlaylistSnapshot,
  StatusSnapshot,
  StreamErrorEvent,
  TrackSnapshot,
};
