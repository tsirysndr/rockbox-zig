// 09 — Browse the local filesystem
//
// Walk `music_dir` like a tree and print directories and files, demonstrating
// the `isDirectory()` helper.
//
//   bun run examples/09-browse-filesystem.ts          # browse music_dir root
//   bun run examples/09-browse-filesystem.ts /Music   # browse a specific path

import { isDirectory } from '../src/index.js';
import { createClient } from './_client.js';

const client = createClient();
const path = process.argv[2];

const entries = await client.browse.entries(path);
console.log(`Browsing: ${path ?? '(music_dir root)'}\n`);

const dirs  = entries.filter(isDirectory);
const files = entries.filter((e) => !isDirectory(e));

console.log(`📁 Directories (${dirs.length}):`);
for (const d of dirs.slice(0, 15)) {
  console.log(`   ${d.displayName ?? d.name}`);
}
if (dirs.length > 15) console.log(`   ... and ${dirs.length - 15} more`);

console.log(`\n🎵 Files (${files.length}):`);
for (const f of files.slice(0, 15)) {
  console.log(`   ${f.displayName ?? f.name}`);
}
if (files.length > 15) console.log(`   ... and ${files.length - 15} more`);
