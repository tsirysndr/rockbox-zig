/**
 * Subsonic / Navidrome HTTP client.
 *
 * Auth scheme: MD5 token auth as per Subsonic API 1.13+
 *   t = md5(password + salt)
 *   s = random 10-char salt
 */

import md5 from "md5";

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
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 10000);
  try {
    const res = await fetch(url, { signal: controller.signal });
    if (!res.ok) return null;
    const json = (await res.json()) as SubsonicResponse;
    const body = json["subsonic-response"];
    if (body?.status !== "ok") return null;
    return body;
  } catch {
    return null;
  } finally {
    clearTimeout(timer);
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
