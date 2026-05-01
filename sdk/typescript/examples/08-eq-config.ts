// 08 — Equalizer & replaygain configuration
//
// Reads the current settings, prints a summary, then applies a "warm + bass
// boost" 5-band EQ preset and enables album-mode ReplayGain.
//
//   bun run examples/08-eq-config.ts

import { ReplaygainType } from '../src/index.js';
import { createClient } from './_client.js';

const client = createClient();

const before = await client.settings.get();
console.log(`Before:`);
console.log(`  EQ enabled       : ${before.eqEnabled}`);
console.log(`  EQ precut        : ${before.eqPrecut}`);
console.log(`  Replaygain type  : ${before.replaygainSettings.type}`);
console.log(`  Replaygain noclip: ${before.replaygainSettings.noclip}`);

await client.settings.save({
  eqEnabled: true,
  eqPrecut: -3,
  eqBandSettings: [
    { cutoff: 60,    q: 7, gain:  4 },  // bass boost
    { cutoff: 200,   q: 7, gain:  1 },
    { cutoff: 800,   q: 7, gain:  0 },
    { cutoff: 4_000, q: 7, gain: -2 },  // tame harshness
    { cutoff: 12_000, q: 7, gain:  2 }, // air
  ],
  replaygainSettings: {
    noclip: true,
    type: ReplaygainType.Album,
    preamp: 0,
  },
});

const after = await client.settings.get();
console.log(`\nAfter:`);
console.log(`  EQ enabled       : ${after.eqEnabled}`);
console.log(`  EQ precut        : ${after.eqPrecut}`);
console.log(`  Replaygain type  : ${after.replaygainSettings.type}`);
console.log(`  Replaygain noclip: ${after.replaygainSettings.noclip}`);
