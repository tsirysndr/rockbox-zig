// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

type SystemStatus = {
  resumeIndex: number;
  resumeCrc32: number;
  resumeElapsed: number;
  resumeOffset: number;
  runtime: number;
  topruntime: number;
  dircacheSize: number;
  lastScreen: number;
  viewerIconCount: number;
  lastVolumeChange: number;
};

declare interface System {
  getGlobalStatus(): Promise<SystemStatus>;
  getRockboxVersion(): Promise<string>;
}
