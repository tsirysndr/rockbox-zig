# `rockbox-rpc` — Expo native module

Expo Modules wrapper around the [`rockbox-expo`](../../../crates/expo/) Rust
crate. Exposes the rockboxd gRPC client and the mDNS discovery surface to
the React Native app under one TypeScript API.

```ts
import RockboxRpc from "rockbox-rpc";
// or via the helper that adds an `isAvailable` guard:
import { RockboxClient } from "@/lib/rockbox-client";
```

This module is autolinked into [`expo/`](../..) via
`"rockbox-rpc": "file:./modules/rockbox-rpc"` in `expo/package.json`. A
`postinstall` hook (`expo/scripts/link-rockbox-rpc.js`) replaces the
copied directory in `node_modules/` with a live symlink so edits to the
TypeScript / Swift / Kotlin source show up immediately.

## Layout

```
modules/rockbox-rpc/
├── expo-module.config.json   declares iOS + Android module classes
├── package.json              local "file:" dependency
├── src/index.ts              TypeScript surface (types + native interface)
├── ios/
│   ├── RockboxRpc.podspec    vendored xcframework + ExpoModulesCore dep
│   └── RockboxRpcModule.swift  @_silgen_name extern declarations + bindings
├── android/
│   ├── build.gradle          jniLibs.srcDirs wired to src/main/jniLibs
│   └── src/main/java/expo/modules/rockboxrpc/RockboxRpcModule.kt
└── scripts/
    ├── build-ios.sh          cross-compile + lipo + xcframework bundle
    └── build-android.sh      cargo-ndk → jniLibs/<abi>/librockbox_expo.so
```

## Building the native libs

You only need to run these once per code change in `crates/expo/`:

```sh
# iOS — produces ios/RockboxExpo.xcframework
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
bun run build:ios

# Android — produces android/src/main/jniLibs/<abi>/librockbox_expo.so
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
export ANDROID_NDK_HOME=...   # NDK r25+
bun run build:android
```

After the artifacts are in place, run `bunx expo prebuild` and then
`bunx expo run:ios` / `run:android` from the `expo/` root.

## API surface

### Init / health

```ts
RockboxClient.setServerUrl("http://192.168.1.10:6061"); // call once at startup
await RockboxClient.ping();                              // → true on success
```

`isAvailable` is `false` on web / before the native libs are built; the JS
helper falls back to throwing a friendly error so the rest of the app can
keep using the mock `PlayerProvider`.

### Playback

```ts
await RockboxClient.play();
await RockboxClient.pause();
await RockboxClient.playPause();
await RockboxClient.next();
await RockboxClient.prev();
await RockboxClient.seek(positionMs);

await RockboxClient.playTrack("/Music/foo.flac");
await RockboxClient.playAlbum(albumId, /* shuffle */ true);
await RockboxClient.playArtistTracks(artistId, false);
await RockboxClient.playAllTracks();
await RockboxClient.playDirectory(path, false, /* position = -1 means "no override" */ -1);
```

### Queue

```ts
await RockboxClient.jumpToQueuePosition(idx);
await RockboxClient.insertTracks(paths, /* position */ -3, /* shuffle */ false);
await RockboxClient.insertTrackNext(path);
await RockboxClient.insertTrackLast(path);
await RockboxClient.insertDirectory(path, position);
await RockboxClient.removeFromQueue(pos);
await RockboxClient.shufflePlaylistAtStart();

const snapshot = await RockboxClient.getPlaylistCurrent();
```

### Library / search

```ts
const tracks  = await RockboxClient.getTracks();
const artists = await RockboxClient.getArtists();
const album   = await RockboxClient.getAlbum(id);
const liked   = await RockboxClient.getLikedTracks();
const results = await RockboxClient.search("aphex twin");

await RockboxClient.likeTrack(trackId);
await RockboxClient.unlikeTrack(trackId);
```

### Sound / settings

```ts
await RockboxClient.adjustVolume(+2);
const v = await RockboxClient.soundCurrent(/* SOUND_VOLUME = 0 */ 0);

await RockboxClient.saveShuffle(true);
await RockboxClient.saveRepeat(/* off=0 all=1 one=2 */ 1);

const status   = await RockboxClient.getGlobalStatus();
const settings = await RockboxClient.getGlobalSettings();
```

### Saved / smart playlists

```ts
const playlists = await RockboxClient.getSavedPlaylists();
await RockboxClient.createSavedPlaylist("Faves", "best of 2024", [trackId1, trackId2]);
await RockboxClient.updateSavedPlaylist(id, "New name", null);
await RockboxClient.deleteSavedPlaylist(id);
await RockboxClient.addTrackToPlaylist(playlistId, trackId);
await RockboxClient.removeTrackFromPlaylist(playlistId, trackId);
const trackIds = await RockboxClient.getSavedPlaylistTracks(playlistId);
await RockboxClient.playSavedPlaylist(playlistId);

const smart = await RockboxClient.getSmartPlaylists();
await RockboxClient.playSmartPlaylist(id);
```

### Browse + Bluetooth

```ts
const root = await RockboxClient.treeGetEntries(null);
const sub  = await RockboxClient.treeGetEntries("/Music/Albums");

if (await RockboxClient.bluetoothAvailable()) {
  const devices = await RockboxClient.getBluetoothDevices();
  await RockboxClient.connectBluetooth(address);
  await RockboxClient.disconnectBluetooth(address);
}
```

### Streaming subscriptions

Each helper returns a single `() => void` that tears down both the event
listener and the native subscription:

```ts
const unsubStatus = RockboxClient.subscribeStatus(
  (s) => setStatus(s.status),
  (e) => console.warn("status stream error", e.error),
);

const unsubTrack = RockboxClient.subscribeCurrentTrack((t) => setTrack(t));
const unsubQueue = RockboxClient.subscribePlaylist((p) => setQueue(p.tracks));
const unsubLib   = RockboxClient.subscribeLibrary((snapshot) => /* refresh */ null);

// LAN scan — defaults to "_rockbox._tcp.local."; pass any other Bonjour name
// for Chromecast etc.
const unsubScan = RockboxClient.subscribeDiscovery(
  (svc) => console.log("found", svc.name, svc.addresses, svc.port),
  undefined,
  RockboxClient.chromecastServiceName(),
);

// later:
unsubStatus(); unsubTrack(); unsubQueue(); unsubLib(); unsubScan();
```

Topics surfaced through the EventEmitter base class:

| Topic | Payload |
|-------|---------|
| `rockbox.status`       | `{ status: 0|1|2 }` |
| `rockbox.currentTrack` | `TrackSnapshot` |
| `rockbox.playlist`     | `PlaylistSnapshot` (`index`, `amount`, `tracks`) |
| `rockbox.library`      | full library snapshot (`unknown` — cast as needed) |
| `rockbox.discovery`    | `DiscoveredService` (`name`, `hostname`, `port`, `addresses[]`, `properties{}`) |
| `rockbox.error`        | `{ subId, stream, error }` |

## Adding a new method

1. Add the `rb_<name>` wrapper in [`crates/expo/src/lib.rs`](../../../crates/expo/src/lib.rs).
2. Declare the symbol on both platforms:
   - Swift (`ios/RockboxRpcModule.swift`): `@_silgen_name(...) private func ...`
   - Kotlin (`android/src/main/java/.../RockboxRpcModule.kt`): `@JvmStatic external fun ...`
3. Add a `Function` / `AsyncFunction` binding in each platform module.
4. Add the typed method to `src/index.ts` (`RockboxRpcNative`) and a
   one-line forwarder on `RockboxClient` in `expo/lib/rockbox-client.ts`.
5. Rebuild the native libs (`bun run build:ios` / `build:android`).

## Troubleshooting

- **`requireNativeModule("RockboxRpc")` fails** — the native libs aren't
  built. Run `bun run build:ios` / `build:android` (and re-prebuild the
  iOS / Android projects).
- **`UnsatisfiedLinkError: librockbox_expo.so`** — the `.so` for the
  current ABI is missing from `android/src/main/jniLibs/`. Make sure
  `cargo-ndk` ran successfully for that ABI.
- **iOS simulator can't find symbols** — the xcframework needs both the
  device and simulator slices. Check that `lipo -info` reports both
  `arm64` and `x86_64` for the simulator library before xcframework packing.
- **Linker error referencing `_dispatch_main_q`** — link `c++` and `resolv`
  (already declared in the podspec).
- **TypeScript can't find `rockbox-rpc`** — bun copied the dir instead of
  symlinking it; the `postinstall` hook replaces it. Re-run `bun install`.

See [`crates/expo/README.md`](../../../crates/expo/README.md) for the
full list of FFI symbols and the Rust-side conventions.
