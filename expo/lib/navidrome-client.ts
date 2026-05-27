/**
 * Subsonic / Navidrome HTTP client — pure TypeScript, no native deps.
 *
 * Auth scheme: MD5 token auth as per Subsonic API 1.13+
 *   t = md5(password + salt)
 *   s = random 10-char salt
 */

// ── Inline MD5 (RFC 1321) ────────────────────────────────────────────────────

function md5(str: string): string {
  function safeAdd(x: number, y: number) {
    const lsw = (x & 0xffff) + (y & 0xffff);
    return (((x >> 16) + (y >> 16) + (lsw >> 16)) << 16) | (lsw & 0xffff);
  }
  function rol(n: number, c: number) { return (n << c) | (n >>> (32 - c)); }
  function cmn(q: number, a: number, b: number, x: number, s: number, t: number) {
    return safeAdd(rol(safeAdd(safeAdd(a, q), safeAdd(x, t)), s), b);
  }
  function ff(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn((b & c) | (~b & d), a, b, x, s, t);
  }
  function gg(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn((b & d) | (c & ~d), a, b, x, s, t);
  }
  function hh(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn(b ^ c ^ d, a, b, x, s, t);
  }
  function ii(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
    return cmn(c ^ (b | ~d), a, b, x, s, t);
  }
  function str2bin(s: string): number[] {
    const b: number[] = [];
    for (let i = 0; i < s.length * 8; i += 8)
      b[i >> 5] = (b[i >> 5] ?? 0) | ((s.charCodeAt(i / 8) & 0xff) << (i % 32));
    return b;
  }
  function bin2hex(b: number[]): string {
    const h = "0123456789abcdef";
    let s = "";
    for (let i = 0; i < b.length * 4; i++)
      s += h[(b[i >> 2] >> ((i % 4) * 8 + 4)) & 0xf] + h[(b[i >> 2] >> ((i % 4) * 8)) & 0xf];
    return s;
  }
  function compute(x: number[], len: number): number[] {
    x[len >> 5] = (x[len >> 5] ?? 0) | (0x80 << (len % 32));
    x[(((len + 64) >>> 9) << 4) + 14] = len;
    let a = 1732584193, b = -271733879, c = -1732584194, d = 271733878;
    for (let i = 0; i < x.length; i += 16) {
      const [oa, ob, oc, od] = [a, b, c, d];
      const W = (j: number) => x[i + j] ?? 0;
      a = ff(a,b,c,d,W(0),7,-680876936);   d=ff(d,a,b,c,W(1),12,-389564586);
      c = ff(c,d,a,b,W(2),17,606105819);   b=ff(b,c,d,a,W(3),22,-1044525330);
      a = ff(a,b,c,d,W(4),7,-176418897);   d=ff(d,a,b,c,W(5),12,1200080426);
      c = ff(c,d,a,b,W(6),17,-1473231341); b=ff(b,c,d,a,W(7),22,-45705983);
      a = ff(a,b,c,d,W(8),7,1770035416);   d=ff(d,a,b,c,W(9),12,-1958414417);
      c = ff(c,d,a,b,W(10),17,-42063);     b=ff(b,c,d,a,W(11),22,-1990404162);
      a = ff(a,b,c,d,W(12),7,1804603682);  d=ff(d,a,b,c,W(13),12,-40341101);
      c = ff(c,d,a,b,W(14),17,-1502002290);b=ff(b,c,d,a,W(15),22,1236535329);
      a = gg(a,b,c,d,W(1),5,-165796510);   d=gg(d,a,b,c,W(6),9,-1069501632);
      c = gg(c,d,a,b,W(11),14,643717713);  b=gg(b,c,d,a,W(0),20,-373897302);
      a = gg(a,b,c,d,W(5),5,-701558691);   d=gg(d,a,b,c,W(10),9,38016083);
      c = gg(c,d,a,b,W(15),14,-660478335); b=gg(b,c,d,a,W(4),20,-405537848);
      a = gg(a,b,c,d,W(9),5,568446438);    d=gg(d,a,b,c,W(14),9,-1019803690);
      c = gg(c,d,a,b,W(3),14,-187363961);  b=gg(b,c,d,a,W(8),20,1163531501);
      a = gg(a,b,c,d,W(13),5,-1444681467); d=gg(d,a,b,c,W(2),9,-51403784);
      c = gg(c,d,a,b,W(7),14,1735328473);  b=gg(b,c,d,a,W(12),20,-1926607734);
      a = hh(a,b,c,d,W(5),4,-378558);      d=hh(d,a,b,c,W(8),11,-2022574463);
      c = hh(c,d,a,b,W(11),16,1839030562); b=hh(b,c,d,a,W(14),23,-35309556);
      a = hh(a,b,c,d,W(1),4,-1530992060);  d=hh(d,a,b,c,W(4),11,1272893353);
      c = hh(c,d,a,b,W(7),16,-155497632);  b=hh(b,c,d,a,W(10),23,-1094730640);
      a = hh(a,b,c,d,W(13),4,681279174);   d=hh(d,a,b,c,W(0),11,-358537222);
      c = hh(c,d,a,b,W(3),16,-722521979);  b=hh(b,c,d,a,W(6),23,76029189);
      a = hh(a,b,c,d,W(9),4,-640364487);   d=hh(d,a,b,c,W(12),11,-421815835);
      c = hh(c,d,a,b,W(15),16,530742520);  b=hh(b,c,d,a,W(2),23,-995338651);
      a = ii(a,b,c,d,W(0),6,-198630844);   d=ii(d,a,b,c,W(7),10,1126891415);
      c = ii(c,d,a,b,W(14),15,-1416354905);b=ii(b,c,d,a,W(5),21,-57434055);
      a = ii(a,b,c,d,W(12),6,1700485571);  d=ii(d,a,b,c,W(3),10,-1894986606);
      c = ii(c,d,a,b,W(10),15,-1051523);   b=ii(b,c,d,a,W(1),21,-2054922799);
      a = ii(a,b,c,d,W(8),6,1873313359);   d=ii(d,a,b,c,W(15),10,-30611744);
      c = ii(c,d,a,b,W(6),15,-1560198380); b=ii(b,c,d,a,W(13),21,1309151649);
      a = ii(a,b,c,d,W(4),6,-145523070);   d=ii(d,a,b,c,W(11),10,-1120210379);
      c = ii(c,d,a,b,W(2),15,718787259);   b=ii(b,c,d,a,W(9),21,-343485551);
      a=safeAdd(a,oa); b=safeAdd(b,ob); c=safeAdd(c,oc); d=safeAdd(d,od);
    }
    return [a, b, c, d];
  }
  const bin = str2bin(str);
  return bin2hex(compute(bin, str.length * 8));
}

// ── Auth helpers ─────────────────────────────────────────────────────────────

function randomSalt(len = 10): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  let s = "";
  for (let i = 0; i < len; i++)
    s += chars[Math.floor(Math.random() * chars.length)];
  return s;
}

export function authParams(user: string, password: string): string {
  const salt = randomSalt();
  const token = md5(password + salt);
  return `u=${encodeURIComponent(user)}&t=${token}&s=${salt}&v=1.16.1&c=rockbox&f=json`;
}

export function coverArtUrl(
  baseUrl: string,
  user: string,
  password: string,
  coverId: string,
  size = 300,
): string {
  const base = baseUrl.replace(/\/$/, "");
  return `${base}/rest/getCoverArt.view?id=${encodeURIComponent(coverId)}&size=${size}&${authParams(user, password)}`;
}

export function streamUrl(
  baseUrl: string,
  user: string,
  password: string,
  songId: string,
): string {
  const base = baseUrl.replace(/\/$/, "");
  return `${base}/rest/stream.view?id=${encodeURIComponent(songId)}&${authParams(user, password)}`;
}

// ── API types ────────────────────────────────────────────────────────────────

export type NdSong = {
  id: string;
  title: string;
  artist: string;
  artistId: string;
  album: string;
  albumId: string;
  coverArt?: string;
  duration: number;
  track?: number;
  streamUrl: string;
};

export type NdAlbum = {
  id: string;
  name: string;
  artist: string;
  artistId: string;
  year?: number;
  coverArt?: string;
  songCount: number;
};

export type NdArtist = {
  id: string;
  name: string;
  coverArt?: string;
  albumCount: number;
};

export type NdPlaylist = {
  id: string;
  name: string;
  comment?: string;
  songCount: number;
  coverArt?: string;
};

// ── Raw Subsonic response types ───────────────────────────────────────────────

type SubsonicResponse = {
  "subsonic-response": {
    status: string;
    error?: { code: number; message: string };
    albumList2?: { album: RawAlbum[] };
    artists?: { index: Array<{ artist: RawArtist[] }> };
    artist?: RawArtist & { album?: RawAlbum[] };
    album?: RawAlbum & { song?: RawSong[] };
    song?: RawSong;
    playlists?: { playlist: RawPlaylist[] };
    playlist?: RawPlaylist & { entry?: RawSong[] };
    starred2?: { song?: RawSong[]; album?: RawAlbum[]; artist?: RawArtist[] };
    searchResult3?: { song?: RawSong[]; album?: RawAlbum[]; artist?: RawArtist[] };
  };
};

type RawAlbum = {
  id: string;
  name: string;
  artist?: string;
  artistId?: string;
  year?: number;
  coverArt?: string;
  songCount?: number;
};

type RawArtist = {
  id: string;
  name: string;
  coverArt?: string;
  albumCount?: number;
  album?: RawAlbum[];
};

type RawSong = {
  id: string;
  title: string;
  artist?: string;
  artistId?: string;
  album?: string;
  albumId?: string;
  coverArt?: string;
  duration?: number;
  track?: number;
};

type RawPlaylist = {
  id: string;
  name: string;
  comment?: string;
  songCount?: number;
  coverArt?: string;
  entry?: RawSong[];
};

// ── Core fetch ────────────────────────────────────────────────────────────────

async function apiFetch(url: string): Promise<SubsonicResponse["subsonic-response"] | null> {
  try {
    const res = await fetch(url, { signal: AbortSignal.timeout(10000) });
    if (!res.ok) return null;
    const json = (await res.json()) as SubsonicResponse;
    const body = json["subsonic-response"];
    if (body.status !== "ok") return null;
    return body;
  } catch {
    return null;
  }
}

function mapSong(raw: RawSong, baseUrl: string, user: string, password: string): NdSong {
  return {
    id: raw.id,
    title: raw.title,
    artist: raw.artist ?? "",
    artistId: raw.artistId ?? "",
    album: raw.album ?? "",
    albumId: raw.albumId ?? "",
    coverArt: raw.coverArt,
    duration: raw.duration ?? 0,
    track: raw.track,
    streamUrl: streamUrl(baseUrl, user, password, raw.id),
  };
}

function mapAlbum(raw: RawAlbum): NdAlbum {
  return {
    id: raw.id,
    name: raw.name,
    artist: raw.artist ?? "",
    artistId: raw.artistId ?? "",
    year: raw.year,
    coverArt: raw.coverArt,
    songCount: raw.songCount ?? 0,
  };
}

function mapArtist(raw: RawArtist): NdArtist {
  return {
    id: raw.id,
    name: raw.name,
    coverArt: raw.coverArt,
    albumCount: raw.albumCount ?? 0,
  };
}

function mapPlaylist(raw: RawPlaylist): NdPlaylist {
  return {
    id: raw.id,
    name: raw.name,
    comment: raw.comment,
    songCount: raw.songCount ?? raw.entry?.length ?? 0,
    coverArt: raw.coverArt,
  };
}

// ── Public API ────────────────────────────────────────────────────────────────

export async function ndGetAlbums(
  baseUrl: string,
  user: string,
  password: string,
  type = "alphabeticalByName",
  size = 500,
): Promise<NdAlbum[]> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getAlbumList2.view?type=${type}&size=${size}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  return (body?.albumList2?.album ?? []).map(mapAlbum);
}

export async function ndGetArtists(
  baseUrl: string,
  user: string,
  password: string,
): Promise<NdArtist[]> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getArtists.view?${authParams(user, password)}`;
  const body = await apiFetch(url);
  const out: NdArtist[] = [];
  for (const index of body?.artists?.index ?? [])
    for (const a of index.artist ?? [])
      out.push(mapArtist(a));
  return out;
}

export async function ndGetAlbum(
  baseUrl: string,
  user: string,
  password: string,
  albumId: string,
): Promise<{ album: NdAlbum; songs: NdSong[] } | null> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getAlbum.view?id=${encodeURIComponent(albumId)}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  if (!body?.album) return null;
  return {
    album: mapAlbum(body.album),
    songs: (body.album.song ?? []).map((s) => mapSong(s, baseUrl, user, password)),
  };
}

export async function ndGetArtist(
  baseUrl: string,
  user: string,
  password: string,
  artistId: string,
): Promise<{ artist: NdArtist; albums: NdAlbum[] } | null> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getArtist.view?id=${encodeURIComponent(artistId)}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  if (!body?.artist) return null;
  return {
    artist: mapArtist(body.artist),
    albums: (body.artist.album ?? []).map(mapAlbum),
  };
}

export async function ndGetSongs(
  baseUrl: string,
  user: string,
  password: string,
): Promise<NdSong[]> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/search3.view?query=&songCount=500&albumCount=0&artistCount=0&${authParams(user, password)}`;
  const body = await apiFetch(url);
  return (body?.searchResult3?.song ?? []).map((s) => mapSong(s, baseUrl, user, password));
}

export async function ndGetPlaylists(
  baseUrl: string,
  user: string,
  password: string,
): Promise<NdPlaylist[]> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getPlaylists.view?${authParams(user, password)}`;
  const body = await apiFetch(url);
  return (body?.playlists?.playlist ?? []).map(mapPlaylist);
}

export async function ndGetPlaylist(
  baseUrl: string,
  user: string,
  password: string,
  playlistId: string,
): Promise<{ playlist: NdPlaylist; songs: NdSong[] } | null> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getPlaylist.view?id=${encodeURIComponent(playlistId)}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  if (!body?.playlist) return null;
  return {
    playlist: mapPlaylist(body.playlist),
    songs: (body.playlist.entry ?? []).map((s) => mapSong(s, baseUrl, user, password)),
  };
}

export async function ndGetStarred(
  baseUrl: string,
  user: string,
  password: string,
): Promise<NdSong[]> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getStarred2.view?${authParams(user, password)}`;
  const body = await apiFetch(url);
  return (body?.starred2?.song ?? []).map((s) => mapSong(s, baseUrl, user, password));
}

export async function ndStar(
  baseUrl: string,
  user: string,
  password: string,
  songId: string,
): Promise<boolean> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/star.view?id=${encodeURIComponent(songId)}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  return body?.status === "ok";
}

export async function ndUnstar(
  baseUrl: string,
  user: string,
  password: string,
  songId: string,
): Promise<boolean> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/unstar.view?id=${encodeURIComponent(songId)}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  return body?.status === "ok";
}

export async function ndScrobble(
  baseUrl: string,
  user: string,
  password: string,
  songId: string,
): Promise<void> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/scrobble.view?id=${encodeURIComponent(songId)}&submission=true&${authParams(user, password)}`;
  await apiFetch(url);
}

export async function ndGetSong(
  baseUrl: string,
  user: string,
  password: string,
  songId: string,
): Promise<NdSong | null> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/getSong.view?id=${encodeURIComponent(songId)}&${authParams(user, password)}`;
  const body = await apiFetch(url);
  if (!body?.song) return null;
  return mapSong(body.song, baseUrl, user, password);
}

export async function ndPing(
  baseUrl: string,
  user: string,
  password: string,
): Promise<boolean> {
  const base = baseUrl.replace(/\/$/, "");
  const url = `${base}/rest/ping.view?${authParams(user, password)}`;
  const body = await apiFetch(url);
  return body?.status === "ok";
}
