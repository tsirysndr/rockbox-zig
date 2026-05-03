# rockbox-now-playing

Native bridge that exposes Android's lock-screen + notification media controls
("now playing card") to the rockboxd remote app. The phone is *not* the audio
source — rockboxd is — so this module deliberately avoids any audio session
plumbing and just renders the controls + forwards transport-button taps back
to JS, where they call the matching gRPC RPCs.

iOS is intentionally not supported in this first pass: the lock-screen card on
iOS only updates while you hold an active audio session, so adding it requires
a silent-audio workaround that's out of scope here.

## Layout

- `src/index.ts` — TypeScript facade (`RockboxNowPlaying`) with `update`,
  `setPlayback`, `clear`, and `onAction`. Falls through to a no-op on iOS / web.
- `android/src/main/.../RockboxNowPlayingModule.kt` — `expo.modules.kotlin`
  module wired up via Expo autolinking. Pushes updates to the foreground
  service via Intents and forwards button taps back to JS as
  `rockbox.nowplaying.action` events.
- `android/src/main/.../NowPlayingService.kt` — foreground service hosting
  the `MediaSessionCompat` and the `MediaStyle` notification. Loads album art
  off the main thread and caches the bitmap until the track changes.

## How JS uses it

`expo/lib/rockbox-streams.tsx` is the single integration point:

```ts
RockboxNowPlaying.update(metadataFor(track, server), {
  isPlaying: status.status === 1,
  positionMs: track.elapsed_ms,
});
RockboxNowPlaying.onAction(({ action, positionMs }) => {
  // Forwards to RockboxClient.play() / pause() / next() / prev() / seek().
});
```

Album art URLs are resolved against the currently-selected server's HTTP
endpoint so the service (running out of process) can fetch them directly
without going through JS.

## Building / running

The module is autolinked through `expo/package.json`'s
`"rockbox-now-playing": "file:./modules/rockbox-now-playing"` entry. Symlinking
into `node_modules/` is handled by `scripts/link-local-modules.js` (postinstall).

The module ships only Kotlin sources — no Rust build step. Just run:

```sh
bunx expo prebuild
bunx expo run:android
```

After native code changes, `bunx expo run:android` from `expo/` is enough.

## Permissions

The module's `AndroidManifest.xml` declares:

- `FOREGROUND_SERVICE` + `FOREGROUND_SERVICE_MEDIA_PLAYBACK` (Android 14+)
- `POST_NOTIFICATIONS` (Android 13+) — without this the OS silently drops the
  notification. The app prompts the user the first time you push a `update`.
- `INTERNET` — for fetching album-art URLs from rockboxd.

## Adding new transport controls

1. Pick or define an `ACTION_BUTTON_*` in `NowPlayingService.kt`.
2. Add the button via `addAction(...)` in `refreshNotification()`.
3. Map the action string in the `onStartCommand` switch.
4. Handle the new string in `dispatchAction` in `lib/now-playing-bridge.ts`.
