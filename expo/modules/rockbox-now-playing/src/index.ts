import { requireNativeModule } from "expo";
import type { EventSubscription } from "expo-modules-core";

/**
 * Snapshot pushed into the lock-screen / notification card. Native side
 * mirrors these fields onto MediaSessionCompat metadata + playback state.
 */
export type NowPlayingMetadata = {
  /** Stable identifier — used to skip re-loading artwork when unchanged. */
  trackId: string;
  title: string;
  artist: string;
  album?: string;
  /** HTTP(S) URL to album art. Loaded on a background thread. */
  artworkUrl?: string | null;
  durationMs: number;
};

/** Playback state pushed independently of metadata so we don't refetch art. */
export type NowPlayingPlayback = {
  isPlaying: boolean;
  positionMs: number;
  /** Mirrors `MediaSessionCompat`'s playback speed — usually 1.0. */
  speed?: number;
};

export type NowPlayingActionEvent = {
  /** Which transport button the user tapped. */
  action: "play" | "pause" | "playPause" | "next" | "prev" | "stop" | "seek";
  /** Populated only when `action === "seek"`. */
  positionMs?: number;
};

type NowPlayingEvents = {
  "rockbox.nowplaying.action": (e: NowPlayingActionEvent) => void;
};

type NowPlayingNative = {
  /** Update both metadata and playback state. Spawns the foreground service if
   *  it isn't already running. Safe to call repeatedly. */
  update(metadata: NowPlayingMetadata, playback: NowPlayingPlayback): void;
  /** Update playback state only — cheap, called on every tick. */
  setPlayback(playback: NowPlayingPlayback): void;
  /** Tear down the notification + service. Call when there's no current track
   *  or the user signs out. */
  clear(): void;
  addListener<K extends keyof NowPlayingEvents>(
    name: K,
    cb: NowPlayingEvents[K],
  ): EventSubscription;
};

let cachedNative: NowPlayingNative | null | undefined;

function getNative(): NowPlayingNative | null {
  if (cachedNative !== undefined) return cachedNative;
  try {
    cachedNative = requireNativeModule<NowPlayingNative>("RockboxNowPlaying");
  } catch {
    cachedNative = null;
  }
  return cachedNative;
}

/**
 * Thin facade so JS callers don't have to null-check the native module on
 * platforms where it isn't shipped (iOS / web today).
 */
export const RockboxNowPlaying = {
  get isAvailable() {
    return getNative() !== null;
  },
  update(metadata: NowPlayingMetadata, playback: NowPlayingPlayback) {
    getNative()?.update(metadata, playback);
  },
  setPlayback(playback: NowPlayingPlayback) {
    getNative()?.setPlayback(playback);
  },
  clear() {
    getNative()?.clear();
  },
  /** Returns an unsubscribe function. */
  onAction(cb: (e: NowPlayingActionEvent) => void): () => void {
    const native = getNative();
    if (!native) return () => {};
    const sub = native.addListener("rockbox.nowplaying.action", cb);
    return () => sub.remove();
  },
};
