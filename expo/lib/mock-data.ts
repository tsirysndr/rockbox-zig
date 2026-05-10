import type { Album, Artist, FileEntry, Playlist, Track } from "./types";

const cover = (seed: string, size = 600) =>
  `https://picsum.photos/seed/${encodeURIComponent(seed)}/${size}/${size}`;

export const ALBUMS: Album[] = [
  { id: "a1", title: "Neon Tides", artist: "Lumen", year: 2024, artwork: cover("neon-tides"), genre: "g5" },
  { id: "a2", title: "Sapphire", artist: "Aerialist", year: 2023, artwork: cover("sapphire"), genre: "g4" },
  { id: "a3", title: "After Hours", artist: "Midnight Bloom", year: 2022, artwork: cover("after-hours"), genre: "g8" },
  { id: "a4", title: "Velvet Sky", artist: "Halo Drift", year: 2024, artwork: cover("velvet-sky"), genre: "g5" },
  { id: "a5", title: "Polaris", artist: "Northern Lights", year: 2021, artwork: cover("polaris"), genre: "g4" },
  { id: "a6", title: "Crystalline", artist: "Glasswave", year: 2023, artwork: cover("crystalline"), genre: "g5" },
  { id: "a7", title: "Echoes", artist: "Hollow Rooms", year: 2022, artwork: cover("echoes"), genre: "g3" },
  { id: "a8", title: "Solstice", artist: "Ember & Oak", year: 2024, artwork: cover("solstice"), genre: "g4" },
];

export const ARTISTS: Artist[] = [
  { id: "ar1", name: "Lumen", image: cover("lumen-artist"), followers: "1.2M" },
  { id: "ar2", name: "Aerialist", image: cover("aerialist"), followers: "842K" },
  { id: "ar3", name: "Midnight Bloom", image: cover("midnight-bloom"), followers: "560K" },
  { id: "ar4", name: "Halo Drift", image: cover("halo-drift"), followers: "390K" },
  { id: "ar5", name: "Northern Lights", image: cover("northern-lights"), followers: "1.4M" },
  { id: "ar6", name: "Glasswave", image: cover("glasswave"), followers: "210K" },
];

export const PLAYLISTS: Playlist[] = [
  {
    id: "p1",
    name: "Daily Mix 1",
    description: "Lumen, Aerialist and more",
    artwork: cover("daily-mix-1"),
    trackCount: 50,
  },
  {
    id: "p2",
    name: "Chill Late Night",
    description: "Wind down with mellow grooves",
    artwork: cover("chill-late-night"),
    trackCount: 42,
  },
  {
    id: "p3",
    name: "Synthwave Drive",
    description: "Sunset cruising soundtrack",
    artwork: cover("synthwave-drive"),
    trackCount: 78,
  },
  {
    id: "p4",
    name: "Focus Flow",
    description: "Uninterrupted instrumental focus",
    artwork: cover("focus-flow"),
    trackCount: 65,
  },
  {
    id: "p5",
    name: "Indie Mornings",
    description: "Fresh starts and hot coffee",
    artwork: cover("indie-mornings"),
    trackCount: 38,
  },
  {
    id: "p6",
    name: "Discover Weekly",
    description: "Your weekly mixtape",
    artwork: cover("discover-weekly"),
    trackCount: 30,
  },
];

export const QUEUE: Track[] = [
  { id: "t1", title: "Glass Cathedral", artist: "Lumen", album: "Neon Tides", duration: 224, artwork: cover("neon-tides") },
  { id: "t2", title: "Aurora", artist: "Lumen", album: "Neon Tides", duration: 198, artwork: cover("neon-tides") },
  { id: "t3", title: "Slow Burn", artist: "Aerialist", album: "Sapphire", duration: 245, artwork: cover("sapphire") },
  { id: "t4", title: "Cobalt", artist: "Aerialist", album: "Sapphire", duration: 211, artwork: cover("sapphire") },
  { id: "t5", title: "Silver Lining", artist: "Midnight Bloom", album: "After Hours", duration: 268, artwork: cover("after-hours") },
  { id: "t6", title: "Drift", artist: "Halo Drift", album: "Velvet Sky", duration: 232, artwork: cover("velvet-sky") },
  { id: "t7", title: "Stargazer", artist: "Northern Lights", album: "Polaris", duration: 312, artwork: cover("polaris") },
  { id: "t8", title: "Refractions", artist: "Glasswave", album: "Crystalline", duration: 199, artwork: cover("crystalline") },
  { id: "t9", title: "Hollow", artist: "Hollow Rooms", album: "Echoes", duration: 254, artwork: cover("echoes") },
  { id: "t10", title: "Last Light", artist: "Ember & Oak", album: "Solstice", duration: 287, artwork: cover("solstice") },
  { id: "t11", title: "Underneath", artist: "Lumen", album: "Neon Tides", duration: 201, artwork: cover("neon-tides") },
  { id: "t12", title: "Indigo", artist: "Aerialist", album: "Sapphire", duration: 240, artwork: cover("sapphire") },
];

export const ALL_SONGS: Track[] = QUEUE;

const EXTRA_ALBUM_TRACK_TITLES: Record<string, string[]> = {
  a1: ["Tideline", "Phosphor", "Reverie", "Lighthouse"],
  a2: ["Marine Bloom", "Moonstone", "Distant Call", "Quiet Waves"],
  a3: ["Kerosene", "Postcards", "Embers", "Glass House"],
  a4: ["Cloudbank", "Sundown", "Pale Blue", "Soft Static"],
  a5: ["North", "Compass", "Solar Wind", "Ice Bloom"],
  a6: ["Prism", "Mosaic", "Quartz", "Lattice"],
  a7: ["Hollow", "Brick & Mortar", "Smoke", "Walls"],
  a8: ["First Frost", "Bonfire", "Long Night", "Daybreak"],
};

const ALBUM_TRACKS: Record<string, Track[]> = {};

for (const album of ALBUMS) {
  const baseTracks = ALL_SONGS.filter((t) => t.album === album.title);
  const extras = (EXTRA_ALBUM_TRACK_TITLES[album.id] ?? []).map(
    (title, i): Track => ({
      id: `${album.id}-x${i}`,
      title,
      artist: album.artist,
      album: album.title,
      duration: 180 + ((i * 47 + album.id.charCodeAt(1)) % 140),
      artwork: album.artwork,
    }),
  );
  ALBUM_TRACKS[album.id] = [...baseTracks, ...extras];
}

export function getAlbumById(id: string): Album | undefined {
  return ALBUMS.find((a) => a.id === id);
}

export function getAlbumTracks(id: string): Track[] {
  return ALBUM_TRACKS[id] ?? [];
}

export function getArtistById(id: string): Artist | undefined {
  return ARTISTS.find((a) => a.id === id);
}

export function getArtistAlbums(artistId: string): Album[] {
  const artist = getArtistById(artistId);
  if (!artist) return [];
  return ALBUMS.filter((a) => a.artist === artist.name);
}

export function getArtistTracks(artistId: string): Track[] {
  const artist = getArtistById(artistId);
  if (!artist) return [];
  const out: Track[] = [];
  for (const album of getArtistAlbums(artistId)) {
    out.push(...getAlbumTracks(album.id));
  }
  return out;
}

const ALL_TRACKS_POOL: Track[] = (() => {
  const seen = new Set<string>();
  const out: Track[] = [];
  for (const t of ALL_SONGS) {
    if (!seen.has(t.id)) {
      seen.add(t.id);
      out.push(t);
    }
  }
  for (const album of ALBUMS) {
    for (const t of getAlbumTracks(album.id)) {
      if (!seen.has(t.id)) {
        seen.add(t.id);
        out.push(t);
      }
    }
  }
  return out;
})();

const PLAYLIST_TRACKS: Record<string, Track[]> = {};

for (const playlist of PLAYLISTS) {
  const seed = playlist.id
    .split("")
    .reduce((acc, ch) => (acc * 31 + ch.charCodeAt(0)) >>> 0, 7);
  const count = Math.min(
    ALL_TRACKS_POOL.length,
    Math.max(8, Math.min(playlist.trackCount, 14)),
  );
  const ordered = [...ALL_TRACKS_POOL];
  // Deterministic Fisher–Yates with the playlist seed
  let s = seed;
  const next = () => (s = (s * 1664525 + 1013904223) >>> 0);
  for (let i = ordered.length - 1; i > 0; i--) {
    const j = next() % (i + 1);
    [ordered[i], ordered[j]] = [ordered[j], ordered[i]];
  }
  PLAYLIST_TRACKS[playlist.id] = ordered.slice(0, count);
}

export function getPlaylistById(id: string): Playlist | undefined {
  return PLAYLISTS.find((p) => p.id === id);
}

export function getPlaylistTracks(id: string): Track[] {
  return PLAYLIST_TRACKS[id] ?? [];
}

export function getGenreById(id: string) {
  return GENRES.find((g) => g.id === id);
}

export function getGenreAlbums(id: string): Album[] {
  return ALBUMS.filter((a) => a.genre === id);
}

export function getGenreArtists(id: string): Artist[] {
  const albums = getGenreAlbums(id);
  const names = new Set(albums.map((a) => a.artist));
  return ARTISTS.filter((a) => names.has(a.name));
}

export function getGenreTracks(id: string): Track[] {
  const albums = getGenreAlbums(id);
  const out: Track[] = [];
  for (const album of albums) {
    out.push(...getAlbumTracks(album.id));
  }
  return out;
}

export const LIKED_TRACK_IDS = new Set(["t1", "t3", "t6", "t9"]);

export const RECENTLY_PLAYED: Album[] = ALBUMS.slice(0, 6);
export const MADE_FOR_YOU: Playlist[] = PLAYLISTS;
export const TOP_ARTISTS: Artist[] = ARTISTS;

export const GENRES = [
  { id: "g1", name: "Pop", color: "#E13300" },
  { id: "g2", name: "Hip-Hop", color: "#1E3264" },
  { id: "g3", name: "Rock", color: "#8400E7" },
  { id: "g4", name: "Indie", color: "#27856A" },
  { id: "g5", name: "Electronic", color: "#1192AA" },
  { id: "g6", name: "Jazz", color: "#A56752" },
  { id: "g7", name: "Classical", color: "#477D95" },
  { id: "g8", name: "R&B", color: "#DC148C" },
  { id: "g9", name: "Country", color: "#777777" },
  { id: "g10", name: "Latin", color: "#E91429" },
];

// ── Files / filesystem mock data ─────────────────────────────────────────────

const MUSIC_ROOT = "/storage/emulated/0/Music";

const LOCAL_FS: Record<string, FileEntry[]> = {
  [MUSIC_ROOT]: [
    { name: "Lumen", path: `${MUSIC_ROOT}/Lumen`, is_dir: true },
    { name: "Aerialist", path: `${MUSIC_ROOT}/Aerialist`, is_dir: true },
    { name: "Midnight Bloom", path: `${MUSIC_ROOT}/Midnight Bloom`, is_dir: true },
    { name: "Halo Drift", path: `${MUSIC_ROOT}/Halo Drift`, is_dir: true },
    { name: "Northern Lights", path: `${MUSIC_ROOT}/Northern Lights`, is_dir: true },
    { name: "Glasswave", path: `${MUSIC_ROOT}/Glasswave`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Lumen`]: [
    { name: "Neon Tides", path: `${MUSIC_ROOT}/Lumen/Neon Tides`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Lumen/Neon Tides`]: [
    { name: "01 - Glass Cathedral.flac", path: `${MUSIC_ROOT}/Lumen/Neon Tides/01 - Glass Cathedral.flac`, is_dir: false },
    { name: "02 - Aurora.flac", path: `${MUSIC_ROOT}/Lumen/Neon Tides/02 - Aurora.flac`, is_dir: false },
    { name: "03 - Underneath.flac", path: `${MUSIC_ROOT}/Lumen/Neon Tides/03 - Underneath.flac`, is_dir: false },
    { name: "04 - Tideline.flac", path: `${MUSIC_ROOT}/Lumen/Neon Tides/04 - Tideline.flac`, is_dir: false },
    { name: "05 - Phosphor.flac", path: `${MUSIC_ROOT}/Lumen/Neon Tides/05 - Phosphor.flac`, is_dir: false },
  ],
  [`${MUSIC_ROOT}/Aerialist`]: [
    { name: "Sapphire", path: `${MUSIC_ROOT}/Aerialist/Sapphire`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Aerialist/Sapphire`]: [
    { name: "01 - Slow Burn.mp3", path: `${MUSIC_ROOT}/Aerialist/Sapphire/01 - Slow Burn.mp3`, is_dir: false },
    { name: "02 - Cobalt.mp3", path: `${MUSIC_ROOT}/Aerialist/Sapphire/02 - Cobalt.mp3`, is_dir: false },
    { name: "03 - Indigo.mp3", path: `${MUSIC_ROOT}/Aerialist/Sapphire/03 - Indigo.mp3`, is_dir: false },
    { name: "04 - Marine Bloom.mp3", path: `${MUSIC_ROOT}/Aerialist/Sapphire/04 - Marine Bloom.mp3`, is_dir: false },
  ],
  [`${MUSIC_ROOT}/Midnight Bloom`]: [
    { name: "After Hours", path: `${MUSIC_ROOT}/Midnight Bloom/After Hours`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Midnight Bloom/After Hours`]: [
    { name: "01 - Silver Lining.mp3", path: `${MUSIC_ROOT}/Midnight Bloom/After Hours/01 - Silver Lining.mp3`, is_dir: false },
    { name: "02 - Kerosene.mp3", path: `${MUSIC_ROOT}/Midnight Bloom/After Hours/02 - Kerosene.mp3`, is_dir: false },
    { name: "03 - Postcards.mp3", path: `${MUSIC_ROOT}/Midnight Bloom/After Hours/03 - Postcards.mp3`, is_dir: false },
  ],
  [`${MUSIC_ROOT}/Halo Drift`]: [
    { name: "Velvet Sky", path: `${MUSIC_ROOT}/Halo Drift/Velvet Sky`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Halo Drift/Velvet Sky`]: [
    { name: "01 - Drift.flac", path: `${MUSIC_ROOT}/Halo Drift/Velvet Sky/01 - Drift.flac`, is_dir: false },
    { name: "02 - Cloudbank.flac", path: `${MUSIC_ROOT}/Halo Drift/Velvet Sky/02 - Cloudbank.flac`, is_dir: false },
    { name: "03 - Sundown.flac", path: `${MUSIC_ROOT}/Halo Drift/Velvet Sky/03 - Sundown.flac`, is_dir: false },
  ],
  [`${MUSIC_ROOT}/Northern Lights`]: [
    { name: "Polaris", path: `${MUSIC_ROOT}/Northern Lights/Polaris`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Northern Lights/Polaris`]: [
    { name: "01 - Stargazer.flac", path: `${MUSIC_ROOT}/Northern Lights/Polaris/01 - Stargazer.flac`, is_dir: false },
    { name: "02 - North.flac", path: `${MUSIC_ROOT}/Northern Lights/Polaris/02 - North.flac`, is_dir: false },
    { name: "03 - Compass.flac", path: `${MUSIC_ROOT}/Northern Lights/Polaris/03 - Compass.flac`, is_dir: false },
  ],
  [`${MUSIC_ROOT}/Glasswave`]: [
    { name: "Crystalline", path: `${MUSIC_ROOT}/Glasswave/Crystalline`, is_dir: true },
  ],
  [`${MUSIC_ROOT}/Glasswave/Crystalline`]: [
    { name: "01 - Refractions.mp3", path: `${MUSIC_ROOT}/Glasswave/Crystalline/01 - Refractions.mp3`, is_dir: false },
    { name: "02 - Prism.mp3", path: `${MUSIC_ROOT}/Glasswave/Crystalline/02 - Prism.mp3`, is_dir: false },
    { name: "03 - Mosaic.mp3", path: `${MUSIC_ROOT}/Glasswave/Crystalline/03 - Mosaic.mp3`, is_dir: false },
  ],
};

export const MOCK_MUSIC_ROOT = MUSIC_ROOT;

export function getLocalEntries(path: string): FileEntry[] {
  return LOCAL_FS[path] ?? [];
}

// UPnP / DLNA mock data
export const UPNP_DEVICES: FileEntry[] = [
  { name: "Living Room NAS (Plex)", path: "upnp://uuid:a1b2c3d4-plex/browse", is_dir: true },
  { name: "Bedroom Pi (Jellyfin)", path: "upnp://uuid:e5f6a7b8-jelly/browse", is_dir: true },
  { name: "NAS (miniDLNA)", path: "upnp://uuid:c9d0e1f2-dlna/browse", is_dir: true },
];

const UPNP_CONTENT: Record<string, FileEntry[]> = {
  "upnp://uuid:a1b2c3d4-plex/browse": [
    { name: "Music", path: "upnp://uuid:a1b2c3d4-plex/music", is_dir: true },
    { name: "Playlists", path: "upnp://uuid:a1b2c3d4-plex/playlists", is_dir: true },
  ],
  "upnp://uuid:a1b2c3d4-plex/music": [
    { name: "Albums", path: "upnp://uuid:a1b2c3d4-plex/music/albums", is_dir: true },
    { name: "Artists", path: "upnp://uuid:a1b2c3d4-plex/music/artists", is_dir: true },
  ],
  "upnp://uuid:a1b2c3d4-plex/music/albums": [
    { name: "Neon Tides — Lumen", path: "upnp://uuid:a1b2c3d4-plex/music/albums/neon-tides", is_dir: true },
    { name: "Sapphire — Aerialist", path: "upnp://uuid:a1b2c3d4-plex/music/albums/sapphire", is_dir: true },
  ],
  "upnp://uuid:a1b2c3d4-plex/music/albums/neon-tides": [
    { name: "Glass Cathedral.flac", path: "upnp://uuid:a1b2c3d4-plex/track/nt-01", is_dir: false },
    { name: "Aurora.flac", path: "upnp://uuid:a1b2c3d4-plex/track/nt-02", is_dir: false },
    { name: "Underneath.flac", path: "upnp://uuid:a1b2c3d4-plex/track/nt-03", is_dir: false },
  ],
  "upnp://uuid:e5f6a7b8-jelly/browse": [
    { name: "Music Library", path: "upnp://uuid:e5f6a7b8-jelly/music", is_dir: true },
  ],
  "upnp://uuid:e5f6a7b8-jelly/music": [
    { name: "After Hours — Midnight Bloom", path: "upnp://uuid:e5f6a7b8-jelly/music/after-hours", is_dir: true },
    { name: "Velvet Sky — Halo Drift", path: "upnp://uuid:e5f6a7b8-jelly/music/velvet-sky", is_dir: true },
  ],
  "upnp://uuid:c9d0e1f2-dlna/browse": [
    { name: "All Music", path: "upnp://uuid:c9d0e1f2-dlna/all", is_dir: true },
  ],
};

export function getUpnpEntries(path: string): FileEntry[] {
  return UPNP_CONTENT[path] ?? [];
}

export function formatDuration(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60)
    .toString()
    .padStart(2, "0");
  return `${m}:${s}`;
}
