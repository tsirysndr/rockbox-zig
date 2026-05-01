// 14 — Plugin: minimal scrobbler
//
// Records a "play" for the previous track when a new track starts, but only if
// the previous one played for at least 30 seconds (Last.fm's classic rule).
// Stats are stored on the rockbox server via `recordTrackPlayed` and feed the
// smart-playlist rules engine.
//
//   bun run examples/14-plugin-scrobbler.ts

import type { RockboxPlugin, Track } from '../src/index.js';
import { createClient } from './_client.js';

const Scrobbler: RockboxPlugin = {
  name: 'scrobbler',
  version: '1.0.0',
  description: 'Record played tracks after 30s of playback',

  install({ events, query }) {
    let current: Track | null = null;
    let startedAt = 0;

    events.on('track:changed', async (track) => {
      const playedFor = current ? Date.now() - startedAt : 0;
      if (current?.id && playedFor >= 30_000) {
        console.log(`✓ scrobbling: ${current.title} — ${current.artist}  (${Math.round(playedFor / 1000)}s)`);
        try {
          await query<{ recordTrackPlayed: boolean }>(
            `mutation Played($id: String!) { recordTrackPlayed(trackId: $id) }`,
            { id: current.id },
          );
        } catch (err) {
          console.error('  scrobble failed:', err instanceof Error ? err.message : err);
        }
      }
      current = track;
      startedAt = Date.now();
      console.log(`▶ now playing: ${track.title} — ${track.artist}`);
    });
  },
};

const client = createClient();
client.connect();
await client.use(Scrobbler);

console.log('Scrobbler installed. Press Ctrl+C to exit.');

process.on('SIGINT', async () => {
  await client.unuse('scrobbler');
  client.disconnect();
  process.exit(0);
});
