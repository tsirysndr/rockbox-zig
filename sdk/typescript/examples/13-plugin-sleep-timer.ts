// 13 — Plugin: sleep timer
//
// Stops playback after N minutes. If the user stops playback manually before
// the timer fires, the plugin cancels itself.
//
//   bun run examples/13-plugin-sleep-timer.ts          # default 30 minutes
//   bun run examples/13-plugin-sleep-timer.ts 5        # 5 minutes

import type { RockboxPlugin } from '../src/index.js';
import { PlaybackStatus } from '../src/index.js';
import { createClient } from './_client.js';

function sleepTimer(minutes: number): RockboxPlugin {
  let timer: ReturnType<typeof setTimeout> | null = null;

  return {
    name: 'sleep-timer',
    version: '1.0.0',
    description: `Stop playback after ${minutes} minute(s)`,

    install({ events, query }) {
      const fireAt = new Date(Date.now() + minutes * 60_000);
      console.log(`💤 Sleep timer armed — will stop playback at ${fireAt.toLocaleTimeString()}`);

      timer = setTimeout(async () => {
        console.log('💤 Time’s up — stopping playback.');
        await query('mutation { hardStop }');
      }, minutes * 60_000);

      // If the user already pressed stop manually, cancel the timer.
      events.on('status:changed', (status) => {
        if (status === PlaybackStatus.Stopped && timer) {
          clearTimeout(timer);
          timer = null;
          console.log('💤 Playback stopped manually — sleep timer cancelled.');
        }
      });
    },

    uninstall() {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    },
  };
}

const minutes = process.argv[2] ? Number(process.argv[2]) : 30;
const client = createClient();
client.connect();

await client.use(sleepTimer(minutes));

console.log('Plugin installed. Press Ctrl+C to cancel and exit.');

process.on('SIGINT', async () => {
  await client.unuse('sleep-timer');
  client.disconnect();
  process.exit(0);
});
