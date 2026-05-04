/**
 * Unified data layer for Library / Home / Search / detail screens.
 *
 * Backed by real proto-typed gRPC reads from `rockbox-rpc`. When no server is
 * selected (`useIsConnected()` is false) the hooks return empty arrays —
 * components are expected to render a "connect to a server" empty state.
 */
import { useMemo } from "react";

import { useIsConnected } from "@/lib/connection";
import type {
  ProtoAlbum,
  ProtoArtist,
  ProtoGenre,
  ProtoPlaylist,
  ProtoTrack,
} from "@/lib/proto-mappers";
import {
  albumFromProto,
  artistFromProto,
  genreFromProto,
  playlistFromProto,
  trackFromProto,
} from "@/lib/proto-mappers";
import {
  useAlbum,
  useAlbums,
  useArtist,
  useArtists,
  useGenre,
  useGenreAlbums,
  useGenreArtists,
  useGenreTracks,
  useGenres,
  useLikedTracks,
  usePlaylistCurrent,
  useSavedPlaylists,
  useSavedPlaylistTracks,
  useSearch,
  useSmartPlaylists,
  useSmartPlaylistTracks,
  useTracks,
} from "@/lib/queries";
import type { Album, Artist, Genre, Playlist, Track } from "@/lib/types";

// ── Top-level lists ─────────────────────────────────────────────────────────

export function useLibraryTracks() {
  const isConnected = useIsConnected();
  const q = useTracks<{ tracks?: ProtoTrack[] }>({ enabled: isConnected });
  const data: Track[] = useMemo(
    () => (isConnected ? (q.data?.tracks ?? []).map(trackFromProto) : []),
    [isConnected, q.data],
  );
  return { data, isLoading: isConnected && q.isLoading, isConnected };
}

/**
 * Liked tracks, in the order returned by the server — `repo::favourites::all_tracks`
 * sorts by `favourites.created_at DESC`, so the most recently liked track is at
 * the top. Don't re-sort client-side.
 */
export function useLibraryLikedTracks() {
  const isConnected = useIsConnected();
  const q = useLikedTracks<{ tracks?: ProtoTrack[] }>({ enabled: isConnected });
  const data: Track[] = useMemo(
    () => (isConnected ? (q.data?.tracks ?? []).map(trackFromProto) : []),
    [isConnected, q.data],
  );
  return { data, isLoading: isConnected && q.isLoading, isConnected };
}

export function useLibraryAlbums() {
  const isConnected = useIsConnected();
  const q = useAlbums<{ albums?: ProtoAlbum[] }>({ enabled: isConnected });
  const data: Album[] = useMemo(
    () => (isConnected ? (q.data?.albums ?? []).map(albumFromProto) : []),
    [isConnected, q.data],
  );
  return { data, isLoading: isConnected && q.isLoading, isConnected };
}

export function useLibraryArtists() {
  const isConnected = useIsConnected();
  const q = useArtists<{ artists?: ProtoArtist[] }>({ enabled: isConnected });
  const data: Artist[] = useMemo(
    () => (isConnected ? (q.data?.artists ?? []).map(artistFromProto) : []),
    [isConnected, q.data],
  );
  return { data, isLoading: isConnected && q.isLoading, isConnected };
}

export function useLibraryPlaylists() {
  const isConnected = useIsConnected();
  const saved = useSavedPlaylists<{ playlists?: ProtoPlaylist[] }>({
    enabled: isConnected,
  });
  const smart = useSmartPlaylists<{ playlists?: ProtoPlaylist[] }>({
    enabled: isConnected,
  });
  const data: Playlist[] = useMemo(() => {
    if (!isConnected) return [];
    const out: Playlist[] = [];
    for (const p of saved.data?.playlists ?? []) out.push(playlistFromProto(p));
    for (const p of smart.data?.playlists ?? [])
      out.push({ ...playlistFromProto(p), isSmart: true });
    return out;
  }, [isConnected, saved.data, smart.data]);
  return {
    data,
    isLoading: isConnected && (saved.isLoading || smart.isLoading),
    isConnected,
  };
}

// ── Detail lookups ─────────────────────────────────────────────────────────

type ProtoAlbumDetail = {
  album?: ProtoAlbum & { tracks?: ProtoTrack[] };
  tracks?: ProtoTrack[];
};

export function useAlbumDetail(id: string) {
  const isConnected = useIsConnected();
  const q = useAlbum<ProtoAlbumDetail>(id, { enabled: isConnected });
  return useMemo(() => {
    if (!isConnected) {
      return {
        album: undefined as Album | undefined,
        tracks: [] as Track[],
        isLoading: false,
        isConnected: false,
      };
    }
    const inner = q.data?.album;
    const album = inner ? albumFromProto(inner) : undefined;
    const tracks =
      q.data?.tracks?.map(trackFromProto) ??
      inner?.tracks?.map(trackFromProto) ??
      [];
    return { album, tracks, isLoading: q.isLoading, isConnected: true };
  }, [isConnected, q.data, q.isLoading]);
}

type ProtoArtistDetail = {
  artist?: ProtoArtist & { albums?: ProtoAlbum[]; tracks?: ProtoTrack[] };
  albums?: ProtoAlbum[];
  tracks?: ProtoTrack[];
};

export function useArtistDetail(id: string) {
  const isConnected = useIsConnected();
  const q = useArtist<ProtoArtistDetail>(id, { enabled: isConnected });
  return useMemo(() => {
    if (!isConnected) {
      return {
        artist: undefined as Artist | undefined,
        albums: [] as Album[],
        tracks: [] as Track[],
        isLoading: false,
        isConnected: false,
      };
    }
    const inner = q.data?.artist;
    const artist = inner ? artistFromProto(inner) : undefined;
    const albums = (q.data?.albums ?? inner?.albums ?? []).map(albumFromProto);
    const tracks = (q.data?.tracks ?? inner?.tracks ?? []).map(trackFromProto);
    return { artist, albums, tracks, isLoading: q.isLoading, isConnected: true };
  }, [isConnected, q.data, q.isLoading]);
}

/**
 * Resolve playlist metadata + tracks. The proto layer doesn't have a single
 * `getPlaylist(id)` RPC — instead we look the playlist up in the saved /
 * smart lists (already cached by `useLibraryPlaylists`) and fetch its track
 * list via the matching `getSavedPlaylistTracks` / `getSmartPlaylistTracks`
 * call. Track ids come back as strings, which we resolve against the
 * library tracks cache to get full Track objects.
 */
export function usePlaylistDetail(id: string) {
  const isConnected = useIsConnected();
  const { data: playlists } = useLibraryPlaylists();
  const { data: tracksLib } = useLibraryTracks();
  const playlist = playlists.find((p) => p.id === id);
  const isSmart = playlist?.isSmart === true;

  const savedTracksQ = useSavedPlaylistTracks<{
    track_ids?: string[];
    tracks?: ProtoTrack[];
  }>(id, { enabled: isConnected && !!playlist && !isSmart });
  const smartTracksQ = useSmartPlaylistTracks<{
    track_ids?: string[];
    tracks?: ProtoTrack[];
  }>(id, { enabled: isConnected && !!playlist && isSmart });
  const tracksQ = isSmart ? smartTracksQ : savedTracksQ;

  return useMemo(() => {
    if (!isConnected) {
      return {
        playlist: undefined as Playlist | undefined,
        tracks: [] as Track[],
        isLoading: false,
        isConnected: false,
      };
    }
    // Server may return the full track protos OR just an array of ids; handle
    // both — when only ids come back, look them up in the library cache.
    const inlineTracks = tracksQ.data?.tracks?.map(trackFromProto);
    const ids = tracksQ.data?.track_ids ?? [];
    const idTracks = inlineTracks
      ? inlineTracks
      : (ids
          .map((tid) => tracksLib.find((t) => t.id === tid))
          .filter((t): t is Track => !!t));
    return {
      playlist,
      tracks: idTracks,
      isLoading: tracksQ.isLoading,
      isConnected: true,
    };
  }, [isConnected, playlist, tracksQ.data, tracksQ.isLoading, tracksLib]);
}

// ── Genres ─────────────────────────────────────────────────────────────────

export function useLibraryGenres() {
  const isConnected = useIsConnected();
  const q = useGenres<{ genres?: ProtoGenre[] }>({ enabled: isConnected });
  const data: Genre[] = useMemo(
    () => (isConnected ? (q.data?.genres ?? []).map(genreFromProto) : []),
    [isConnected, q.data],
  );
  return {
    data,
    isLoading: isConnected && q.isLoading,
    isConnected,
    error: q.error,
  };
}

type ProtoGenreDetail = { genre?: ProtoGenre };
type ProtoGenreTracks = { tracks?: ProtoTrack[] };
type ProtoGenreAlbums = { albums?: ProtoAlbum[] };
type ProtoGenreArtists = { artists?: ProtoArtist[] };

export function useGenreDetail(id: string) {
  const isConnected = useIsConnected();
  const enabled = isConnected && id.length > 0;
  const meta = useGenre<ProtoGenreDetail>(id, { enabled });
  const tracksQ = useGenreTracks<ProtoGenreTracks>(id, { enabled });
  const albumsQ = useGenreAlbums<ProtoGenreAlbums>(id, { enabled });
  const artistsQ = useGenreArtists<ProtoGenreArtists>(id, { enabled });

  return useMemo(() => {
    if (!isConnected) {
      return {
        genre: undefined as Genre | undefined,
        tracks: [] as Track[],
        albums: [] as Album[],
        artists: [] as Artist[],
        isLoading: false,
        isConnected: false,
      };
    }
    const inner = meta.data?.genre;
    const genre = inner ? genreFromProto(inner) : undefined;
    const tracks = (tracksQ.data?.tracks ?? []).map(trackFromProto);
    const albums = (albumsQ.data?.albums ?? []).map(albumFromProto);
    const artists = (artistsQ.data?.artists ?? []).map(artistFromProto);
    return {
      genre,
      tracks,
      albums,
      artists,
      isLoading:
        meta.isLoading ||
        tracksQ.isLoading ||
        albumsQ.isLoading ||
        artistsQ.isLoading,
      isConnected: true,
    };
  }, [
    isConnected,
    meta.data,
    meta.isLoading,
    tracksQ.data,
    tracksQ.isLoading,
    albumsQ.data,
    albumsQ.isLoading,
    artistsQ.data,
    artistsQ.isLoading,
  ]);
}

// ── Search ─────────────────────────────────────────────────────────────────

type ProtoSearchResults = {
  tracks?: ProtoTrack[];
  albums?: ProtoAlbum[];
  artists?: ProtoArtist[];
  playlists?: ProtoPlaylist[];
};

export type SearchResults = {
  tracks: Track[];
  albums: Album[];
  artists: Artist[];
  playlists: Playlist[];
};

export function useLibrarySearch(term: string) {
  const isConnected = useIsConnected();
  const q = useSearch<ProtoSearchResults>(term, { enabled: isConnected });
  const data: SearchResults = useMemo(() => {
    if (!isConnected || !q.data) {
      return { tracks: [], albums: [], artists: [], playlists: [] };
    }
    return {
      tracks: (q.data.tracks ?? []).map(trackFromProto),
      albums: (q.data.albums ?? []).map(albumFromProto),
      artists: (q.data.artists ?? []).map(artistFromProto),
      playlists: (q.data.playlists ?? []).map(playlistFromProto),
    };
  }, [isConnected, q.data]);
  return { data, isLoading: isConnected && q.isLoading, isConnected };
}

// ── Current queue ──────────────────────────────────────────────────────────

type ProtoQueue = {
  index?: number;
  amount?: number;
  tracks?: ProtoTrack[];
};

export function useCurrentQueue() {
  const isConnected = useIsConnected();
  const q = usePlaylistCurrent<ProtoQueue>({ enabled: isConnected });
  return useMemo(() => {
    if (!isConnected) {
      return {
        tracks: [] as Track[],
        currentIdx: 0,
        isLoading: false,
        isConnected: false,
      };
    }
    const tracks = (q.data?.tracks ?? []).map(trackFromProto);
    return {
      tracks,
      currentIdx: Math.max(0, q.data?.index ?? 0),
      isLoading: q.isLoading,
      isConnected: true,
    };
  }, [isConnected, q.data, q.isLoading]);
}
