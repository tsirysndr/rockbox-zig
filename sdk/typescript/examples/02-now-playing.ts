// 02 — Now playing (real-time subscriptions)
//
// Opens a WebSocket and prints every track / status / queue change as it
// happens. Press Ctrl+C to exit.
//
//   bun run examples/02-now-playing.ts

import { PlaybackStatus } from '../src/index.js';
import { createClient } from './_client.js';

const client = createClient();
client.connect();

client.on('track:changed', (track) => {
  console.log(`▶  ${track.title} — ${track.artist}  [${track.album}]`);
});

client.on('status:changed', (raw) => {
  console.log(`◐  ${PlaybackStatus[raw] ?? raw}`);
});

client.on('playlist:changed', (queue) => {
  console.log(`☰  queue updated — ${queue.amount} tracks (index ${queue.index})`);
});

client.on('ws:error', (err) => {
  console.error(`✗  websocket error: ${err.message}`);
});

console.log('Listening for events. Press Ctrl+C to exit.');

// Tear down cleanly on Ctrl+C
process.on('SIGINT', () => {
  console.log('\nDisconnecting...');
  client.disconnect();
  process.exit(0);
});
