# Rockbox Mobile

A React Native / Expo client for Rockbox — the mobile companion to the
[GPUI desktop app](../gpui). Designed with a Spotify / Tidal-inspired layout
on top of the Rockbox dark theme (`#0F1117` background, `#6F00FF` accent).

## Stack

- **Expo Router** — file-based routing (`app/`)
- **NativeWind** — Tailwind utility classes for React Native
- **expo-image** — fast image loading for album art
- **expo-font** — bundles `SpaceGrotesk` (UI) and `JetBrainsMono` (numerals)
- **@expo/vector-icons** — Ionicons for playback controls

## Layout

```
app/
├─ _layout.tsx          Root stack — fonts, PlayerProvider, modals
├─ (tabs)/
│  ├─ _layout.tsx       Custom tab bar with persistent MiniPlayer
│  ├─ index.tsx         Home — greeting, quick picks, recently played, mixes
│  ├─ search.tsx        Search bar + genre tile grid + live results
│  └─ library.tsx       Playlists / Songs / Albums / Artists / Liked pills
├─ player.tsx           Full-screen now playing (modal, slide-up)
└─ queue.tsx            Up-next queue (modal)

components/
├─ mini-player.tsx      Pinned bottom miniplayer with progress bar
├─ seek-bar.tsx         Tappable / draggable progress + volume slider
├─ card-row.tsx         Horizontal scroll of square / circular tiles
└─ section-header.tsx   Section title + subtitle

lib/
├─ player-context.tsx   Global playback state (queue, position, like, repeat…)
├─ mock-data.ts         Albums / artists / playlists / tracks / genres
└─ types.ts             Shared types

constants/theme.ts      Rockbox dark palette (mirrors gpui/src/ui/theme.rs)
tailwind.config.js      Same palette exposed as Tailwind colors
```

## Dark theme

Mirrors [`gpui/src/ui/theme.rs`](../gpui/src/ui/theme.rs):

| Token            | Hex                      |
| ---------------- | ------------------------ |
| `appBg`          | `#0F1117`                |
| `bgCard`         | `#1A1D26`                |
| `accent`         | `#6F00FF`                |
| `accentSoft`     | `#1A0E3D`                |
| `text/primary`   | `#FFFFFF`                |
| `text/secondary` | `#9898A8`                |
| `border`         | `rgba(255,255,255,0.16)` |

The app icon (`assets/images/icon.png`, splash, favicon, Android adaptive
foreground) is rendered from [`gpui/assets/rockbox.svg`](../gpui/assets/rockbox.svg).

## Get started

```sh
bun install   # or npm install / yarn
bun run start # or npx expo start
```

Then pick:

- `i` — iOS Simulator
- `a` — Android emulator
- `w` — Web (works for layout review; native-only modules will degrade gracefully)

## Mock playback

The `PlayerProvider` runs a 1 Hz tick that advances `position` while
`isPlaying` is true, with auto-advance and repeat-one / repeat-all
semantics that mirror the GPUI controller. There is no audio engine wired
up yet — controls update the in-memory state only. To connect real
playback, replace the action handlers in `lib/player-context.tsx` with
calls to the rockboxd gRPC / GraphQL API.
