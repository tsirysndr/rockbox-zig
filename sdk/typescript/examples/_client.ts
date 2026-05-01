// Shared client factory used by every example.
// Override the host/port via env: ROCKBOX_HOST, ROCKBOX_PORT.
import { RockboxClient } from '../src/index.js';

export function createClient(): RockboxClient {
  return new RockboxClient({
    host: process.env.ROCKBOX_HOST ?? 'localhost',
    port: process.env.ROCKBOX_PORT ? Number(process.env.ROCKBOX_PORT) : 6062,
  });
}

/** Format milliseconds as M:SS */
export function fmtTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}
