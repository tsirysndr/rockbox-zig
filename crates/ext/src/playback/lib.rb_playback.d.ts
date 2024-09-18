// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

type Mp3Entry = {};

declare interface Playback {
  play(elapsed: number, offset: number): Promise<void>;
  pause(): Promise<void>;
  resume(): Promise<void>;
  next(): Promise<void>;
  previous(): Promise<void>;
  fastForwardRewind(): Promise<void>;
  status(): Promise<number>;
  currentTrack(): Promise<Mp3Entry | null>;
  nextTrack(): Promise<Mp3Entry | null>;
  flushAndReloadTracks(): Promise<void>;
  getFilePosition(): Promise<number>;
  hardStop(): Promise<void>;
}
