// 11 — Bluetooth (Linux only)
//
// List paired devices, optionally scan for new ones, and connect/disconnect by
// MAC address. Works on Linux hosts where rockboxd has access to BlueZ.
//
//   bun run examples/11-bluetooth.ts                    # list paired devices
//   bun run examples/11-bluetooth.ts scan               # scan 10s for devices
//   bun run examples/11-bluetooth.ts connect AA:BB:..   # connect by address
//   bun run examples/11-bluetooth.ts disconnect AA:BB:..

import type { BluetoothDevice } from '../src/index.js';
import { createClient } from './_client.js';

function format(d: BluetoothDevice): string {
  const flags = [
    d.connected ? 'connected' : '',
    d.paired    ? 'paired'    : '',
    d.trusted   ? 'trusted'   : '',
  ].filter(Boolean).join(', ');
  const rssi = d.rssi != null ? ` ${d.rssi} dBm` : '';
  return `  ${d.address}  ${d.name.padEnd(28)}${rssi}  [${flags}]`;
}

const client = createClient();
const cmd = process.argv[2] ?? 'list';

try {
  if (cmd === 'scan') {
    console.log('Scanning for 10s...');
    const found = await client.bluetooth.scan(10);
    console.log(`Found ${found.length} devices:`);
    found.forEach((d) => console.log(format(d)));
  } else if (cmd === 'connect' && process.argv[3]) {
    await client.bluetooth.connect(process.argv[3]);
    console.log(`Connected to ${process.argv[3]}`);
  } else if (cmd === 'disconnect' && process.argv[3]) {
    await client.bluetooth.disconnect(process.argv[3]);
    console.log(`Disconnected from ${process.argv[3]}`);
  } else {
    const devices = await client.bluetooth.devices();
    console.log(`Paired devices (${devices.length}):`);
    devices.forEach((d) => console.log(format(d)));
  }
} catch (err) {
  // Non-Linux hosts return "Bluetooth is only supported on Linux".
  console.error('Bluetooth call failed:', err instanceof Error ? err.message : err);
  process.exit(1);
}
