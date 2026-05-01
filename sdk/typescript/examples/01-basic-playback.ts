// 01 — Basic playback
//
// Inspect the current track, then either pause or resume based on the current
// state. Idempotent: run it twice and it toggles between Playing and Paused.
//
//   bun run examples/01-basic-playback.ts

import { PlaybackStatus } from "../src/index.js";
import { createClient, fmtTime } from "./_client.js";

const client = createClient();

const status = await client.playback.status();
console.log(`Status: ${PlaybackStatus[status]}`);

const track = await client.playback.currentTrack();
if (track) {
  const pct =
    track.length > 0 ? Math.round((track.elapsed / track.length) * 100) : 0;
  console.log(`Now: ${track.title} — ${track.artist}`);
  console.log(
    `     ${fmtTime(track.elapsed)} / ${fmtTime(track.length)} (${pct}%)`,
  );
} else {
  console.log("Nothing is playing.");
}
// Toggle playback
if (status === PlaybackStatus.Playing) {
  await client.playback.pause();
  console.log("→ paused");
} else if (status === PlaybackStatus.Paused) {
  await client.playback.resume();
  console.log("→ resumed");
}
