// 05 — Saved playlists
//
// Create a new playlist, populate it with a few tracks from the library, list
// existing playlists, then clean up after itself.
//
//   bun run examples/05-saved-playlists.ts

import { createClient } from './_client.js';

const client = createClient();

// 1) Pick the first 3 track IDs from the library
const tracks = await client.library.tracks();
const seedIds = tracks
  .slice(0, 3)
  .map((t) => t.id)
  .filter((id): id is string => Boolean(id));

if (seedIds.length === 0) {
  console.error('No tracks in the library — cannot create a playlist.');
  process.exit(1);
}

// 2) Create the playlist with those tracks
const created = await client.savedPlaylists.create({
  name: `SDK example — ${new Date().toISOString()}`,
  description: 'Demo playlist created by examples/05-saved-playlists.ts',
  trackIds: seedIds,
});
console.log(`Created playlist ${created.id} — "${created.name}" (${created.trackCount} tracks)`);

// 3) Show all playlists
const all = await client.savedPlaylists.list();
console.log(`\nAll playlists (${all.length}):`);
for (const p of all.slice(0, 10)) {
  console.log(`  • ${p.name} — ${p.trackCount} tracks`);
}

// 4) Add one more track, then remove the original first track
if (tracks[3]?.id) {
  await client.savedPlaylists.addTracks(created.id, [tracks[3].id]);
  console.log(`\nAdded one more track`);
}
if (seedIds[0]) {
  await client.savedPlaylists.removeTrack(created.id, seedIds[0]);
  console.log(`Removed the first seed track`);
}

const refreshed = await client.savedPlaylists.get(created.id);
console.log(`Now has ${refreshed?.trackCount ?? 0} tracks`);

// 5) Cleanup — comment this out to keep the demo playlist around
await client.savedPlaylists.delete(created.id);
console.log(`\nCleaned up demo playlist.`);
