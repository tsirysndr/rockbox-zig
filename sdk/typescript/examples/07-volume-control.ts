// 07 — Volume control
//
// Read the current volume (with min/max range) and bump it up by one step.
//
//   bun run examples/07-volume-control.ts          # show + step up
//   bun run examples/07-volume-control.ts -3       # step down 3
//
// Volume is in firmware-defined steps (typically dB on PortalPlayer targets).

import { createClient } from './_client.js';

const client = createClient();
const delta = process.argv[2] ? Number(process.argv[2]) : 1;

const before = await client.sound.getVolume();
const range = before.max - before.min;
const filled = range > 0 ? Math.round(((before.volume - before.min) / range) * 20) : 0;
const bar = '█'.repeat(filled) + '░'.repeat(Math.max(0, 20 - filled));

console.log(`Volume: ${before.volume} dB  (range ${before.min} … ${before.max})`);
console.log(`        ${bar}`);

const after = await client.sound.adjustVolume(delta);
console.log(`\nAdjusted by ${delta >= 0 ? `+${delta}` : delta} → ${after} dB`);
