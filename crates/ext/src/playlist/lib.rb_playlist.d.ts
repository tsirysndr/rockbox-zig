// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

declare interface Playlist {
  getResumeInfo(): Promise<void>;
  getTrackInfo(): Promise<void>;
  getFirstIndex(): Promise<void>;
  getDisplayIndex(): Promise<void>;
  amount(): Promise<void>;
  playlistResume(): Promise<void>;
  resumeTrack(): Promise<void>;
  setModified(): Promise<void>;
  start(): Promise<void>;
  sync(): Promise<void>;
  removeAllTracks(): Promise<void>;
  createPlaylist(): Promise<void>;
  insertTrack(): Promise<void>;
  insertDirectory(): Promise<void>;
  insertPlaylist(): Promise<void>;
  shufflePlaylist(): Promise<void>;
  warnOnPlaylistErase(): Promise<void>;
}
