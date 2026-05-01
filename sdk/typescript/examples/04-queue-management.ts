// 04 — Queue management
//
// Print the current queue, demonstrate inserting tracks at different positions,
// and removing entries.
//
//   bun run examples/04-queue-management.ts

import { InsertPosition } from '../src/index.js';
import { createClient, fmtTime } from './_client.js';

const client = createClient();

const queue = await client.playlist.current();
console.log(`Queue: ${queue.amount} tracks, currently at index ${queue.index}\n`);

queue.tracks.slice(0, 10).forEach((t, i) => {
  const marker = i === queue.index ? '▶' : ' ';
  console.log(`${marker} ${(i + 1).toString().padStart(3)}. ${t.title} — ${t.artist}  (${fmtTime(t.length)})`);
});
if (queue.tracks.length > 10) console.log(`  ... and ${queue.tracks.length - 10} more`);

// --- Append a track at the end of the queue (no playback change) ----------
// Replace this path with one that exists on your system before running.
const PATH_TO_INSERT = process.argv[2];

if (PATH_TO_INSERT) {
  console.log(`\nAppending: ${PATH_TO_INSERT}`);
  await client.playlist.insertTracks([PATH_TO_INSERT], InsertPosition.Last);

  const after = await client.playlist.amount();
  console.log(`→ queue now has ${after} tracks`);
} else {
  console.log('\n(Pass a file path to insert it at the end of the queue.)');
}
