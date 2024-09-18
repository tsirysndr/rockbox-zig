// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

declare interface Sound {
  adjustVolume(): Promise<void>;
  soundSet(): Promise<void>;
  soundCurrent(): Promise<void>;
  soundDefault(): Promise<void>;
  soundMin(): Promise<void>;
  soundMax(): Promise<void>;
  soundUnit(): Promise<void>;
  soundVal2Phys(): Promise<void>;
  getPitch(): Promise<void>;
  setPitch(): Promise<void>;
  beepPlay(): Promise<void>;
  pcmbufFade(): Promise<void>;
  pcmGetLowLatency(): Promise<void>;
  systemSoundPlay(): Promise<void>;
  keyClickClick(): Promise<void>;
}
