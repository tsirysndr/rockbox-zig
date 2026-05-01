// 06 — Smart playlist
//
// Build a smart playlist that resolves to the user's most-played tracks, then
// resolve the rules right now and print the matches.
//
//   bun run examples/06-smart-playlist.ts

import { createClient } from './_client.js';

const client = createClient();

// Smart-playlist rules are server-evaluated and serialized as JSON. The
// schema is documented in the rockbox_playlists::rules module.
const rules = JSON.stringify({
  operator: 'AND',
  rules: [{ field: 'play_count', op: 'gte', value: 1 }],
  sort: { field: 'play_count', dir: 'desc' },
  limit: 25,
});

const sp = await client.smartPlaylists.create({
  name: 'Most played (demo)',
  description: 'Top 25 most-played tracks',
  rules,
});
console.log(`Created smart playlist: ${sp.id}`);

// Resolve the rules right now and inspect the results.
const ids = await client.smartPlaylists.trackIds(sp.id);
console.log(`Currently resolves to ${ids.length} tracks`);

// Show stats for the first match
if (ids[0]) {
  const stats = await client.smartPlaylists.trackStats(ids[0]);
  if (stats) {
    console.log(
      `Top track stats: played ${stats.playCount}× (skipped ${stats.skipCount}×)`,
    );
  }
}

// Cleanup
await client.smartPlaylists.delete(sp.id);
console.log('Cleaned up demo smart playlist.');
