/**
 * Convert proto JSON returned by `rockbox-rpc` into the app's `Track`/
 * `Album`/`Artist` types. Each mapper is total — missing fields default
 * to empty strings / 0 / null so the UI renders a sensible placeholder.
 */
import { coverUrl } from "@/lib/cover-url";
import type { Album, Artist, Genre, Playlist, Track } from "@/lib/types";

export type ProtoTrack = {
  id?: string;
  path?: string;
  title?: string;
  artist?: string;
  album?: string;
  album_id?: string;
  artist_id?: string;
  album_artist?: string;
  genre?: string;
  album_art?: string | null;
  // length is in milliseconds (the proto field name is `length`).
  length?: number;
  track_number?: number;
  disc_number?: number;
};

export type ProtoAlbum = {
  id?: string;
  title?: string;
  artist?: string;
  artist_id?: string;
  album_art?: string | null;
  year?: number;
  year_string?: string;
  copyright_message?: string | null;
  genre?: string;
};

export type ProtoArtist = {
  id?: string;
  name?: string;
  image?: string | null;
  bio?: string;
};

export type ProtoPlaylist = {
  id?: string;
  name?: string;
  description?: string | null;
  image?: string | null;
  track_count?: number;
  is_smart?: boolean;
  rules?: string | null;
};

export type ProtoGenre = {
  id?: string;
  name?: string;
  description?: string | null;
  image?: string | null;
  track_count?: number;
};

const fallback = (s: string | null | undefined, alt = ""): string => s ?? alt;

export function trackFromProto(p: ProtoTrack | undefined | null): Track {
  if (!p) return blankTrack();
  const durationSecs = Math.max(0, Math.floor((p.length ?? 0) / 1000));
  return {
    id: fallback(p.id),
    path: fallback(p.path),
    title: fallback(p.title, "(unknown)"),
    artist: fallback(p.artist),
    artistId: fallback(p.artist_id) || undefined,
    album: fallback(p.album),
    albumId: fallback(p.album_id) || undefined,
    duration: durationSecs,
    artwork: coverUrl(p.album_art) ?? undefined,
    trackNumber: p.track_number ?? undefined,
    discNumber: p.disc_number ?? undefined,
  };
}

export function albumFromProto(p: ProtoAlbum | undefined | null): Album {
  if (!p) return blankAlbum();
  return {
    id: fallback(p.id),
    title: fallback(p.title, "(untitled)"),
    artist: fallback(p.artist),
    artwork: coverUrl(p.album_art) ?? "",
    year: p.year ?? (parseInt(p.year_string ?? "", 10) || undefined),
    yearString: p.year_string && p.year_string.length > 0 ? p.year_string : undefined,
    copyrightMessage:
      p.copyright_message && p.copyright_message.length > 0
        ? p.copyright_message
        : undefined,
    genre: p.genre ?? undefined,
  };
}

export function artistFromProto(p: ProtoArtist | undefined | null): Artist {
  if (!p) return blankArtist();
  return {
    id: fallback(p.id),
    name: fallback(p.name, "(unknown)"),
    // Proto-side artist images aren't populated yet — fall back to empty
    // string and let the UI show a placeholder/initial.
    image: p.image ?? "",
  };
}

export function playlistFromProto(
  p: ProtoPlaylist | undefined | null,
): Playlist {
  if (!p) return blankPlaylist();
  return {
    id: fallback(p.id),
    name: fallback(p.name, "(untitled)"),
    description: p.description ?? undefined,
    artwork: coverUrl(p.image) ?? "",
    trackCount: p.track_count ?? 0,
    isSmart: p.is_smart === true,
    rules: p.rules ?? undefined,
  };
}

export function genreFromProto(p: ProtoGenre | undefined | null): Genre {
  if (!p) return { id: "", name: "", color: "#374151" };
  const id = fallback(p.id);
  const name = fallback(p.name, "(unknown)");
  return { id, name, color: colorForSeed(id || name) };
}

// Deterministic HSL → hex for genre tiles. Mirrors the GPUI/macOS hash:
// FNV-ish * 31 over the seed, then map into the 0–359 hue band.
function colorForSeed(seed: string): string {
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    hash = (Math.imul(hash, 31) + seed.charCodeAt(i)) | 0;
  }
  const hue = ((hash % 360) + 360) % 360;
  return hslToHex(hue, 0.65, 0.45);
}

function hslToHex(h: number, s: number, l: number): string {
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  let r = 0, g = 0, b = 0;
  if (h < 60) { r = c; g = x; }
  else if (h < 120) { r = x; g = c; }
  else if (h < 180) { g = c; b = x; }
  else if (h < 240) { g = x; b = c; }
  else if (h < 300) { r = x; b = c; }
  else { r = c; b = x; }
  const to = (v: number) =>
    Math.round((v + m) * 255).toString(16).padStart(2, "0");
  return `#${to(r)}${to(g)}${to(b)}`;
}

function blankTrack(): Track {
  return {
    id: "",
    path: "",
    title: "",
    artist: "",
    album: "",
    duration: 0,
  };
}
function blankAlbum(): Album {
  return { id: "", title: "", artist: "", artwork: "" };
}
function blankArtist(): Artist {
  return { id: "", name: "", image: "" };
}
function blankPlaylist(): Playlist {
  return { id: "", name: "", artwork: "", trackCount: 0 };
}
