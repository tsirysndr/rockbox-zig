// 10 — Browse UPnP media servers
//
// `treeGetEntries` accepts paths starting with `upnp://` to discover and walk
// UPnP ContentDirectory servers on the local network.
//
//   bun run examples/10-browse-upnp.ts            # discover servers
//   bun run examples/10-browse-upnp.ts <path>     # browse a server / object
//
// Discovered servers come back as entries with names like `upnp://<encoded>`.
// Pass that name back as the path argument to descend into one. The `name`
// field of a child entry is the canonical path you can navigate into next.

import { isDirectory } from '../src/index.js';
import { createClient } from './_client.js';

const client = createClient();
const path = process.argv[2] ?? 'upnp://';

const entries = await client.browse.entries(path);
console.log(`UPnP browse: ${path}`);
console.log(`Found ${entries.length} entries.\n`);

for (const e of entries.slice(0, 30)) {
  const icon = isDirectory(e) ? '📁' : '🎵';
  // displayName carries the human-readable title for UPnP entries
  const label = e.displayName ?? e.name;
  console.log(`${icon} ${label}`);
  console.log(`   → ${e.name}`);
}
if (entries.length > 30) console.log(`\n... and ${entries.length - 30} more`);

if (entries.length === 0) {
  console.log('No UPnP servers responded. Make sure a UPnP/DLNA server is running');
  console.log('on the local network (e.g. minidlna, Plex, Kodi).');
}
