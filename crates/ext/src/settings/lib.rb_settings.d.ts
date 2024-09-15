// deno-lint-ignore-file no-explicit-any

/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

declare interface Settings {
  getGlobalSettings(): Promise<void>;
}
