// 15 — Tiny interactive CLI remote
//
// A no-frills terminal remote control. Renders the now-playing line and
// reacts to single-key presses without needing Enter.
//
//   bun run examples/15-cli-remote.ts
//
// Keys:
//   space  play / pause
//   n      next track
//   p      previous track
//   +      volume up
//   -      volume down
//   l      like current track
//   q      quit

import { PlaybackStatus } from '../src/index.js';
import { createClient, fmtTime } from './_client.js';

const client = createClient();
client.connect();

async function render(): Promise<void> {
  const [status, track, vol] = await Promise.all([
    client.playback.status(),
    client.playback.currentTrack(),
    client.sound.getVolume(),
  ]);

  const label = PlaybackStatus[status] ?? '?';
  const line = track
    ? `${track.title} — ${track.artist}  [${fmtTime(track.elapsed)} / ${fmtTime(track.length)}]`
    : '(nothing playing)';

  process.stdout.write(`\r\x1b[2K[${label}]  ${line}   vol=${vol.volume}dB`);
}

await render();

// Re-render whenever something changes
client.on('track:changed',  () => { void render(); });
client.on('status:changed', () => { void render(); });

// Raw stdin: single keypress without Enter
const stdin = process.stdin;
stdin.setRawMode?.(true);
stdin.resume();
stdin.setEncoding('utf8');

stdin.on('data', async (key) => {
  const k = key.toString();

  if (k === 'q' || k === '' /* Ctrl-C */) {
    stdin.setRawMode?.(false);
    client.disconnect();
    console.log('\nbye');
    process.exit(0);
  }

  try {
    switch (k) {
      case ' ': {
        const status = await client.playback.status();
        if (status === PlaybackStatus.Playing) await client.playback.pause();
        else                                   await client.playback.resume();
        break;
      }
      case 'n': await client.playback.next();      break;
      case 'p': await client.playback.previous();  break;
      case '+':
      case '=': await client.sound.volumeUp();     break;
      case '-':
      case '_': await client.sound.volumeDown();   break;
      case 'l': {
        const t = await client.playback.currentTrack();
        if (t?.id) {
          await client.library.likeTrack(t.id);
          process.stdout.write('  ♥');
        }
        break;
      }
    }
  } catch (err) {
    process.stdout.write(`\n[error] ${err instanceof Error ? err.message : err}\n`);
  }

  await render();
});

console.log('Press [space] play/pause  [n] next  [p] prev  [+/-] vol  [l] like  [q] quit');
