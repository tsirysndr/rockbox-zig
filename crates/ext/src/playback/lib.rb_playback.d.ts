// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

declare interface Playback {
  play(): Promise<void>;
  pause(): Promise<void>;
  resume(): Promise<void>;
  next(): Promise<void>;
  previous(): Promise<void>;
  fastForwardRewind(): Promise<void>;
  status(): Promise<void>;
  currentTrack(): Promise<void>;
  flushAndReload(): Promise<void>;
  getFilePosition(): Promise<void>;
  hardStop(): Promise<void>;
}
