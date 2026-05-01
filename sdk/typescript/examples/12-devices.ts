// 12 — Output devices (Chromecast / AirPlay / source devices)
//
// List discovered output sinks and optionally connect/disconnect one.
//
//   bun run examples/12-devices.ts                      # list devices
//   bun run examples/12-devices.ts connect <id>         # connect a device
//   bun run examples/12-devices.ts disconnect <id>      # disconnect a device

import { createClient } from './_client.js';

const client = createClient();
const cmd = process.argv[2] ?? 'list';

if (cmd === 'connect' && process.argv[3]) {
  await client.devices.connect(process.argv[3]);
  console.log(`Connected to device ${process.argv[3]}`);
} else if (cmd === 'disconnect' && process.argv[3]) {
  await client.devices.disconnect(process.argv[3]);
  console.log(`Disconnected device ${process.argv[3]}`);
} else {
  const devices = await client.devices.list();
  console.log(`Discovered ${devices.length} device(s):\n`);

  for (const d of devices) {
    const kind = d.isCastDevice
      ? 'cast'
      : d.isSourceDevice
      ? 'source'
      : 'output';
    const dot = d.isConnected ? '●' : '○';
    const cur = d.isCurrentDevice ? ' (current)' : '';
    console.log(`${dot} [${kind.padEnd(6)}] ${d.name}${cur}`);
    console.log(`     id=${d.id}  ${d.ip}:${d.port}  ${d.service}`);
    if (d.baseUrl) console.log(`     baseUrl=${d.baseUrl}`);
  }
}
