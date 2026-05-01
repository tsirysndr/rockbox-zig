// 03 — Search the library
//
// Search the library for a term passed on the command line, print a summary,
// and start playing the first matching album (if any).
//
//   bun run examples/03-library-search.ts "pink floyd"

import { createClient } from './_client.js';

const term = process.argv[2];
if (!term) {
  console.error('usage: bun run examples/03-library-search.ts <search term>');
  process.exit(1);
}

const client = createClient();

const { artists, albums, tracks, likedAlbums, likedTracks } = await client.library.search(term);

console.log(`Search: "${term}"`);
console.log(`  Artists      : ${artists.length}`);
console.log(`  Albums       : ${albums.length}`);
console.log(`  Tracks       : ${tracks.length}`);
console.log(`  Liked albums : ${likedAlbums.length}`);
console.log(`  Liked tracks : ${likedTracks.length}\n`);

console.log('Top albums:');
for (const a of albums.slice(0, 5)) {
  const copyright = a.copyrightMessage ? ` © ${a.copyrightMessage}` : '';
  console.log(`  • ${a.title} — ${a.artist} (${a.year})${copyright}`);
}

if (albums[0]) {
  console.log(`\nPlaying: ${albums[0].title}`);
  await client.playback.playAlbum(albums[0].id, { shuffle: false });
}
