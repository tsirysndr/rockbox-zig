# @rockbox-zig/sdk

TypeScript SDK for [Rockbox](https://www.rockbox.org) — a fully typed GraphQL client with real-time subscriptions and a plugin system.

## Table of contents

- [Installation](#installation)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [API reference](#api-reference)
  - [Playback](#playback)
  - [Library](#library)
  - [Playlist (queue)](#playlist-queue)
  - [Saved playlists](#saved-playlists)
  - [Smart playlists](#smart-playlists)
  - [Sound](#sound)
  - [Settings](#settings)
  - [System](#system)
  - [Browse (filesystem)](#browse-filesystem)
  - [Devices](#devices)
- [Real-time events](#real-time-events)
- [Plugin system](#plugin-system)
- [Error handling](#error-handling)
- [Raw GraphQL queries](#raw-graphql-queries)
- [Types reference](#types-reference)

---

## Installation

```sh
bun add @rockbox-zig/sdk
# or
npm install @rockbox-zig/sdk
```

`rockboxd` must be running and reachable. By default the SDK connects to `http://localhost:6062/graphql`. Start rockboxd with:

```sh
./zig/zig-out/bin/rockboxd
```

---

## Quick start

```typescript
import { RockboxClient, PlaybackStatus } from '@rockbox-zig/sdk';

const client = new RockboxClient();

// Optional: start WebSocket subscriptions for real-time events
client.connect();

// Check what is playing
const track = await client.playback.currentTrack();
if (track) {
  console.log(`Now playing: ${track.title} — ${track.artist}`);
}

// Search the library
const { albums, tracks } = await client.library.search('dark side');
console.log(`Found ${albums.length} albums and ${tracks.length} tracks`);

// Play an album with shuffle
await client.playback.playAlbum(albums[0]!.id, { shuffle: true });

// React to track changes
client.on('track:changed', (track) => {
  console.log(`▶ ${track.title} by ${track.artist}`);
});

// Tear down when done
client.disconnect();
```

---

## Configuration

```typescript
import { RockboxClient } from '@rockbox-zig/sdk';

// Defaults: localhost:6062
const client = new RockboxClient();

// Custom host and port
const client = new RockboxClient({ host: '192.168.1.42', port: 6062 });

// Fully custom URLs (useful behind a reverse proxy)
const client = new RockboxClient({
  httpUrl: 'https://music.home/graphql',
  wsUrl:   'wss://music.home/graphql',
});
```

| Option    | Type     | Default                        | Description                                         |
|-----------|----------|--------------------------------|-----------------------------------------------------|
| `host`    | `string` | `"localhost"`                  | Hostname or IP of rockboxd                          |
| `port`    | `number` | `6062`                         | GraphQL port (env: `ROCKBOX_GRAPHQL_PORT`)          |
| `httpUrl` | `string` | `http://{host}:{port}/graphql` | Override the full HTTP URL                          |
| `wsUrl`   | `string` | `ws://{host}:{port}/graphql`   | Override the full WebSocket URL                     |

---

## API reference

### Playback

```typescript
client.playback
```

#### Transport controls

```typescript
// Check status
const status = await client.playback.status();
// Returns a PlaybackStatus enum value
// PlaybackStatus.Stopped = 0, PlaybackStatus.Playing = 1, PlaybackStatus.Paused = 2

import { PlaybackStatus } from '@rockbox-zig/sdk';
if (status === PlaybackStatus.Playing) {
  await client.playback.pause();
} else {
  await client.playback.resume();
}

await client.playback.next();
await client.playback.previous();
await client.playback.stop();

// Seek to an absolute position (milliseconds)
await client.playback.seek(90_000); // jump to 1:30

// Get current file byte offset
const pos = await client.playback.filePosition();
```

#### Current and next track

```typescript
const track = await client.playback.currentTrack();
// null when nothing is playing
if (track) {
  const pct = ((track.elapsed / track.length) * 100).toFixed(0);
  console.log(`${track.title} — ${pct}% (${track.bitrate} kbps)`);
}

const next = await client.playback.nextTrack();
```

#### Play helpers

Single-call shortcuts to load content and start playing immediately:

```typescript
// Play a single file by path
await client.playback.playTrack('/Music/Pink Floyd/Wish You Were Here.mp3');

// Play all tracks from an album
await client.playback.playAlbum('album-id');
await client.playback.playAlbum('album-id', { shuffle: true });
await client.playback.playAlbum('album-id', { position: 3 }); // start at track index 3

// Play all tracks from an artist
await client.playback.playArtist('artist-id', { shuffle: true });

// Play a saved playlist
await client.playback.playPlaylist('playlist-id', { shuffle: true });

// Play all files under a directory (recursively, shuffled)
await client.playback.playDirectory('/Music/Jazz', { recurse: true, shuffle: true });

// Play the liked tracks collection
await client.playback.playLikedTracks({ shuffle: true });

// Play the entire library, shuffled
await client.playback.playAllTracks({ shuffle: true });

// Force-reload the current queue from disk
await client.playback.flushAndReload();
```

---

### Library

```typescript
client.library
```

#### Albums

```typescript
// All albums — tracks array contains shallow track stubs
const albums = await client.library.albums();
albums.forEach((a) => console.log(`${a.title} (${a.year}) — ${a.artist}`));

// Single album with full track list
const album = await client.library.album('album-id');
if (album) {
  console.log(`${album.title} — ${album.tracks.length} tracks`);
  album.tracks.forEach((t) => console.log(`  ${t.tracknum}. ${t.title}`));
}

// Liked albums
const liked = await client.library.likedAlbums();

// Like / unlike
await client.library.likeAlbum('album-id');
await client.library.unlikeAlbum('album-id');
```

#### Artists

```typescript
const artists = await client.library.artists();
artists.forEach((a) => console.log(`${a.name} — ${a.albums.length} albums`));

// Single artist with albums and tracks
const artist = await client.library.artist('artist-id');
if (artist) {
  console.log(`${artist.name}`);
  artist.albums.forEach((a) => console.log(`  • ${a.title} (${a.year})`));
}
```

#### Tracks

```typescript
const tracks = await client.library.tracks();
const track  = await client.library.track('track-id');
const liked  = await client.library.likedTracks();

await client.library.likeTrack('track-id');
await client.library.unlikeTrack('track-id');
```

#### Search

```typescript
const results = await client.library.search('radiohead');
// results: { artists, albums, tracks, likedTracks, likedAlbums }

console.log(`Artists : ${results.artists.map((a) => a.name).join(', ')}`);
console.log(`Albums  : ${results.albums.map((a) => a.title).join(', ')}`);
console.log(`Tracks  : ${results.tracks.length} match(es)`);
```

#### Library scan

```typescript
// Trigger a full rescan of music_dir
await client.library.scan();
```

---

### Playlist (queue)

```typescript
client.playlist
```

The *playlist* API manages the live playback queue — what is playing right now. For persistent named collections see [Saved playlists](#saved-playlists).

```typescript
// Inspect the current queue
const queue = await client.playlist.current();
console.log(`${queue.amount} tracks, at index ${queue.index}`);

queue.tracks.forEach((t, i) => {
  const active = i === queue.index ? '▶' : ' ';
  console.log(`${active} ${i + 1}. ${t.title} — ${t.artist}`);
});

const count = await client.playlist.amount();
```

#### Queue management

```typescript
import { InsertPosition } from '@rockbox-zig/sdk';

// Insert right after the current track
await client.playlist.insertTracks(
  ['/Music/track1.mp3', '/Music/track2.mp3'],
  InsertPosition.Next,
);

// Append at the end
await client.playlist.insertTracks(paths, InsertPosition.Last);

// Replace the whole queue
await client.playlist.insertTracks(paths, InsertPosition.First);

// Insert a whole directory
await client.playlist.insertDirectory('/Music/Ambient', InsertPosition.Last);

// Insert an album by library ID
await client.playlist.insertAlbum('album-id', InsertPosition.Next);

// Remove track at queue index 2 (0-based)
await client.playlist.removeTrack(2);

// Clear the entire queue
await client.playlist.clear();

// Shuffle remaining tracks
await client.playlist.shuffle();

// Create a new queue and start playing immediately
await client.playlist.create('Evening Mix', [
  '/Music/track1.mp3',
  '/Music/track2.mp3',
  '/Music/track3.mp3',
]);

// Resume from where playback was stopped
await client.playlist.resume();
```

| `InsertPosition` | Value | Effect                                   |
|------------------|-------|------------------------------------------|
| `Next`           | `0`   | After the currently playing track        |
| `AfterCurrent`   | `1`   | After the last manually inserted track   |
| `Last`           | `2`   | At the end of the queue                  |
| `First`          | `3`   | Replace the entire queue                 |

---

### Saved playlists

```typescript
client.savedPlaylists
```

Saved playlists are persistent named collections stored in the database.

```typescript
// List all playlists, optionally filtered by folder
const playlists = await client.savedPlaylists.list();
const inFolder  = await client.savedPlaylists.list('folder-id');

// Get a single playlist
const pl = await client.savedPlaylists.get('playlist-id');
if (pl) console.log(`${pl.name} — ${pl.trackCount} tracks`);

// Get ordered track IDs
const trackIds = await client.savedPlaylists.trackIds('playlist-id');

// Create
const newPl = await client.savedPlaylists.create({
  name: 'Late Night Jazz',
  description: 'Quiet music for working',
  folderId: 'folder-id',          // optional
  trackIds: ['t1', 't2', 't3'],   // optional seed tracks
});

// Update metadata
await client.savedPlaylists.update('playlist-id', {
  name: 'Late Night Jazz (updated)',
  description: 'Still quiet',
});

// Add / remove individual tracks
await client.savedPlaylists.addTracks('playlist-id', ['t4', 't5']);
await client.savedPlaylists.removeTrack('playlist-id', 't1');

// Load into queue and play
await client.savedPlaylists.play('playlist-id');

// Delete permanently
await client.savedPlaylists.delete('playlist-id');
```

#### Folders

```typescript
const folders = await client.savedPlaylists.folders();
folders.forEach((f) => console.log(f.name));

const folder = await client.savedPlaylists.createFolder('Work');
console.log(`Created folder: ${folder.id}`);

await client.savedPlaylists.deleteFolder(folder.id);
```

---

### Smart playlists

```typescript
client.smartPlaylists
```

Smart playlists evaluate a rule set dynamically each time they are played.

```typescript
// List and get
const smarts = await client.smartPlaylists.list();
const sp     = await client.smartPlaylists.get('smart-id');

// Resolve the matching track IDs right now
const ids = await client.smartPlaylists.trackIds('smart-id');
console.log(`Smart playlist resolves to ${ids.length} tracks`);

// Create — rules is a JSON-encoded rule set
const recentlyPlayed = await client.smartPlaylists.create({
  name: 'Recently played',
  rules: JSON.stringify({
    operator: 'AND',
    rules: [
      { field: 'play_count', op: 'gt',     value: 0     },
      { field: 'last_played', op: 'within', value: '30d' },
    ],
  }),
});

// Create a "top rated" smart playlist
const topRated = await client.smartPlaylists.create({
  name: 'Most played',
  rules: JSON.stringify({
    operator: 'AND',
    rules: [{ field: 'play_count', op: 'gte', value: 10 }],
    sort: { field: 'play_count', dir: 'desc' },
    limit: 50,
  }),
});

// Update, play, delete
await client.smartPlaylists.update('smart-id', { name: 'Recently played (60d)', rules: '...' });
await client.smartPlaylists.play('smart-id');
await client.smartPlaylists.delete('smart-id');
```

#### Listening stats

Stats power smart playlist rules and are also useful for scrobbling integrations.

```typescript
const stats = await client.smartPlaylists.trackStats('track-id');
if (stats) {
  console.log(`Played: ${stats.playCount}, skipped: ${stats.skipCount}`);
  if (stats.lastPlayed) {
    console.log(`Last played: ${new Date(stats.lastPlayed * 1000).toLocaleDateString()}`);
  }
}

// Record events manually (e.g. from a scrobbler plugin)
await client.smartPlaylists.recordPlayed('track-id');
await client.smartPlaylists.recordSkipped('track-id');
```

---

### Sound

```typescript
client.sound
```

Volume is adjusted in firmware-defined steps (not absolute dB values). The number of steps per dB varies by hardware target.

```typescript
// Relative adjustment: positive = louder, negative = quieter
const newRaw = await client.sound.adjustVolume(+3);  // 3 steps up
await client.sound.adjustVolume(-1);                 // 1 step down

// Convenience one-step helpers
await client.sound.volumeUp();    // equivalent to adjustVolume(+1)
await client.sound.volumeDown();  // equivalent to adjustVolume(-1)
```

---

### Settings

```typescript
client.settings
```

```typescript
// Read all global settings
const s = await client.settings.get();
console.log(`Music directory : ${s.musicDir}`);
console.log(`Volume          : ${s.volume}`);
console.log(`EQ enabled      : ${s.eqEnabled}`);
console.log(`Repeat mode     : ${s.repeatMode}`);

// Partial update — only the fields you pass are written
import { RepeatMode } from '@rockbox-zig/sdk';

await client.settings.save({
  shuffle: true,
  repeatMode: RepeatMode.All,
});

// Enable and configure the equalizer
await client.settings.save({
  eqEnabled: true,
  eqPrecut: -3,
  eqBandSettings: [
    { cutoff: 60,    q: 7, gain:  3 },  // bass boost
    { cutoff: 200,   q: 7, gain:  0 },
    { cutoff: 800,   q: 7, gain:  0 },
    { cutoff: 4000,  q: 7, gain: -2 },  // presence cut
    { cutoff: 12000, q: 7, gain:  1 },
  ],
});

// Configure dynamics compression
await client.settings.save({
  compressorSettings: {
    threshold: -24,
    makeupGain: 3,
    ratio: 2,         // 2:1
    knee: 0,
    releaseTime: 100,
    attackTime: 5,
  },
});

// Replaygain
import { ReplaygainType } from '@rockbox-zig/sdk';
await client.settings.save({
  replaygainSettings: {
    noclip: true,
    type: ReplaygainType.Album,
    preamp: 0,
  },
});
```

---

### System

```typescript
client.system
```

```typescript
const version = await client.system.version();
console.log(`rockboxd ${version}`);

const status = await client.system.status();
console.log(`Runtime  : ${status.runtime}s`);
console.log(`Top run  : ${status.topruntime}s`);
console.log(`Resume at: track ${status.resumeIndex}`);
```

---

### Browse (filesystem)

```typescript
client.browse
```

Walk the filesystem relative to the configured `music_dir`.

```typescript
import { isDirectory } from '@rockbox-zig/sdk';

// Root of music_dir
const entries = await client.browse.entries();

// Specific path
const entries = await client.browse.entries('/Music/Pink Floyd');
entries.forEach((e) => {
  const icon = isDirectory(e) ? '📁' : '🎵';
  console.log(`${icon} ${e.name}`);
});

// Only directories
const dirs = await client.browse.directories('/Music');
dirs.forEach((d) => console.log(d.name));

// Only files
const files = await client.browse.files('/Music/Pink Floyd/The Wall');
files.forEach((f) => console.log(f.name));
```

---

### Devices

```typescript
client.devices
```

Devices are remote output sinks (Chromecast, AirPlay receivers, etc.) discovered via mDNS.

```typescript
// List all discovered devices
const devices = await client.devices.list();
devices.forEach((d) => {
  const status = d.isConnected ? '● connected' : '○ available';
  const type   = d.isCastDevice ? 'Cast' : d.isSourceDevice ? 'Source' : 'Other';
  console.log(`[${type}] ${d.name} (${d.ip}:${d.port}) — ${status}`);
});

// Get a specific device
const device = await client.devices.get('device-id');
if (device?.isCastDevice) {
  console.log(`Casting to: ${device.name}`);
}

// Connect — switches the active PCM output sink to this device
await client.devices.connect('chromecast-device-id');

// Disconnect — reverts to the built-in PCM sink
await client.devices.disconnect('chromecast-device-id');
```

---

## Real-time events

Call `client.connect()` to open the WebSocket. The connection is lazy (only created on first call), auto-reconnecting with exponential backoff up to 30 seconds.

```typescript
import { RockboxClient, PlaybackStatus, type Track } from '@rockbox-zig/sdk';

const client = new RockboxClient();
client.connect();

// ── Track changes ──────────────────────────────────────────────────────────
client.on('track:changed', (track: Track) => {
  document.title = `${track.title} — ${track.artist}`;
  updateNowPlayingUI(track);
});

// ── Playback status ────────────────────────────────────────────────────────
client.on('status:changed', (raw) => {
  const label: Record<number, string> = {
    [PlaybackStatus.Stopped]: 'Stopped',
    [PlaybackStatus.Playing]: 'Playing',
    [PlaybackStatus.Paused]:  'Paused',
  };
  setStatusBadge(label[raw] ?? 'Unknown');
});

// ── Queue changes ──────────────────────────────────────────────────────────
client.on('playlist:changed', (queue) => {
  console.log(`Queue updated — ${queue.amount} tracks`);
  renderQueue(queue.tracks);
});

// ── WebSocket lifecycle ────────────────────────────────────────────────────
client.on('ws:error', (err) => console.error('WebSocket error:', err.message));

// ── One-time listener ──────────────────────────────────────────────────────
client.once('track:changed', (track) => {
  console.log('First track event received:', track.title);
});

// ── Remove a listener ──────────────────────────────────────────────────────
const handler = (track: Track) => console.log(track.title);
client.on('track:changed', handler);
// ...later:
client.off('track:changed', handler);

// ── Tear down subscriptions and close the WebSocket ────────────────────────
client.disconnect();
```

### Event map

| Event              | Payload    | Description                           |
|--------------------|------------|---------------------------------------|
| `track:changed`    | `Track`    | Currently playing track changed       |
| `status:changed`   | `number`   | Playback status changed (0 / 1 / 2)   |
| `playlist:changed` | `Playlist` | Active queue was modified             |
| `ws:error`         | `Error`    | WebSocket or subscription error       |
| `ws:open`          | —          | WebSocket connection established      |
| `ws:close`         | —          | WebSocket connection closed           |

---

## Plugin system

Plugins are the recommended way to add cross-cutting features — scrobbling, notifications, analytics, remote control — without forking the SDK. The design is inspired by Jellyfin's `IPlugin` interface.

### Writing a plugin

```typescript
import type { RockboxPlugin, PluginContext, Track } from '@rockbox-zig/sdk';

export const LastFmScrobbler: RockboxPlugin = {
  name: 'lastfm-scrobbler',
  version: '1.0.0',
  description: 'Scrobble played tracks to Last.fm',

  install({ events }: PluginContext) {
    let startedAt = 0;
    let current: Track | null = null;

    events.on('track:changed', (track) => {
      // Scrobble the previous track if it played for more than 30 s
      if (current && Date.now() - startedAt > 30_000) {
        submitScrobble({ title: current.title, artist: current.artist });
      }
      current = track;
      startedAt = Date.now();
    });
  },

  uninstall() {
    // release any timers or external connections here
  },
};
```

### Installing a plugin

```typescript
const client = new RockboxClient();
client.connect();

await client.use(LastFmScrobbler);

// List what is installed
client.installedPlugins().forEach((p) => {
  console.log(`${p.name} v${p.version}${p.description ? ` — ${p.description}` : ''}`);
});

// Uninstall cleanly
await client.unuse('lastfm-scrobbler');
```

### Plugin with custom queries

`PluginContext.query()` exposes the raw HTTP transport for one-off GraphQL operations:

```typescript
const LyricsPlugin: RockboxPlugin = {
  name: 'lyrics',
  version: '0.1.0',

  install({ query, events }) {
    events.on('track:changed', async (track) => {
      if (!track.id) return;
      const data = await query<{ track: { title: string; artist: string } }>(
        `query T($id: String!) { track(id: $id) { title artist } }`,
        { id: track.id },
      );
      fetchAndDisplayLyrics(data.track.title, data.track.artist);
    });
  },
};
```

### Example: desktop notification plugin

```typescript
import type { RockboxPlugin } from '@rockbox-zig/sdk';

export const DesktopNotifications: RockboxPlugin = {
  name: 'desktop-notifications',
  version: '1.0.0',

  install({ events }) {
    if (typeof Notification === 'undefined') return;

    Notification.requestPermission();

    events.on('track:changed', (track) => {
      new Notification(track.title, {
        body: `${track.artist}  ·  ${track.album}`,
        icon: track.albumArt ?? undefined,
      });
    });
  },
};
```

### Example: auto-scrobble + sleep timer plugin

```typescript
import type { RockboxPlugin } from '@rockbox-zig/sdk';

export function sleepTimer(minutes: number): RockboxPlugin {
  let timer: ReturnType<typeof setTimeout> | null = null;

  return {
    name: 'sleep-timer',
    version: '1.0.0',
    description: `Stop playback after ${minutes} minutes`,

    install({ events, query }) {
      timer = setTimeout(async () => {
        await query('mutation { hardStop }');
        console.log('Sleep timer fired — playback stopped.');
      }, minutes * 60_000);

      events.on('status:changed', (status) => {
        // Cancel the timer if the user already stopped playback manually
        if (status === 0 && timer) {
          clearTimeout(timer);
          timer = null;
        }
      });
    },

    uninstall() {
      if (timer) clearTimeout(timer);
    },
  };
}

// Usage
await client.use(sleepTimer(30)); // stop in 30 minutes
```

---

## Error handling

All methods throw typed errors, making it easy to distinguish network failures from API errors:

```typescript
import {
  RockboxNetworkError,
  RockboxGraphQLError,
  RockboxError,
} from '@rockbox-zig/sdk';

try {
  await client.playback.play();
} catch (err) {
  if (err instanceof RockboxNetworkError) {
    // rockboxd is unreachable — show an "offline" indicator
    showOfflineBanner(err.message);
  } else if (err instanceof RockboxGraphQLError) {
    // The server returned structured GraphQL errors
    for (const e of err.errors) {
      console.error('GraphQL:', e.message, e.path);
    }
  } else if (err instanceof RockboxError) {
    // Base class — catches any remaining SDK error
    console.error('Rockbox error:', err.message);
  }
}
```

| Class                  | When thrown                                              |
|------------------------|----------------------------------------------------------|
| `RockboxNetworkError`  | `fetch` rejects or HTTP status is not 2xx               |
| `RockboxGraphQLError`  | Server returns `{ errors: [...] }` in the response body |
| `RockboxError`         | Base class — catch to handle all SDK errors              |

---

## Raw GraphQL queries

For operations not yet covered by the SDK use the `client.query()` escape hatch. The GraphiQL explorer is available at `http://localhost:6062/graphiql` while rockboxd is running.

```typescript
// Simple query
const data = await client.query<{ rockboxVersion: string }>(
  `query { rockboxVersion }`,
);
console.log(data.rockboxVersion);

// Query with variables
const data = await client.query<{ album: { id: string; title: string } | null }>(
  `query Album($id: String!) {
     album(id: $id) { id title artist year }
   }`,
  { id: 'album-123' },
);

// Mutation
await client.query(
  `mutation Seek($t: Int!) { fastForwardRewind(newTime: $t) }`,
  { t: 120_000 },
);
```

---

## Types reference

All types and enums are exported from `@rockbox-zig/sdk`:

```typescript
import type {
  Track, Album, Artist, SearchResults,
  Playlist, SavedPlaylist, SavedPlaylistFolder,
  SmartPlaylist, TrackStats,
  Device, Entry,
  UserSettings, PartialUserSettings,
  EqBandSetting, ReplaygainSettings, CompressorSettings,
  SystemStatus,
} from '@rockbox-zig/sdk';
```

### Enums

```typescript
import {
  PlaybackStatus,   // Stopped=0  Playing=1  Paused=2
  RepeatMode,       // Off=0  All=1  One=2  Shuffle=3  ABRepeat=4
  ChannelConfig,    // Stereo  StereoNarrow  Mono  LeftMix  RightMix  Karaoke
  ReplaygainType,   // Track=0  Album=1  Shuffle=2
  InsertPosition,   // Next=0  AfterCurrent=1  Last=2  First=3
} from '@rockbox-zig/sdk';
```

### Selected type shapes

```typescript
interface Track {
  id?: string;
  title: string;       artist: string;      album: string;
  genre: string;       albumArtist: string; composer: string;
  tracknum: number;    discnum: number;     year: number;
  bitrate: number;     frequency: number;
  length: number;      // duration in ms
  elapsed: number;     // current position in ms
  filesize: number;    path: string;
  albumId?: string;    artistId?: string;   albumArt?: string;
}

interface Album {
  id: string;   title: string;  artist: string;
  year: number; artistId: string; md5: string;
  albumArt?: string;
  tracks: Track[];
}

interface Playlist {
  amount: number;  index: number;  maxPlaylistSize: number;
  firstIndex: number;  lastInsertPos: number;
  seed: number;  lastShuffledStart: number;
  tracks: Track[];
}

interface Device {
  id: string;      name: string;  host: string;
  ip: string;      port: number;  service: string;
  isConnected: boolean;
  isCastDevice: boolean;  isSourceDevice: boolean;  isCurrentDevice: boolean;
  baseUrl?: string;
}
```

### Helper functions

```typescript
import { isDirectory } from '@rockbox-zig/sdk';

const entries = await client.browse.entries('/Music');
const dirs  = entries.filter(isDirectory);
const files = entries.filter((e) => !isDirectory(e));
```
