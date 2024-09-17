// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

declare interface Playback {
  play(elapsed: number, offset: number): Promise<void>;
  pause(): Promise<void>;
  resume(): Promise<void>;
  next(): Promise<void>;
  previous(): Promise<void>;
  fastForwardRewind(): Promise<void>;
  status(): Promise<void>;
  currentTrack(): Promise<void>;
  flushAndReloadTracks(): Promise<void>;
  getFilePosition(): Promise<void>;
  hardStop(): Promise<void>;
}
