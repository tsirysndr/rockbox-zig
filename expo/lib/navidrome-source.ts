/**
 * React Query hooks for Navidrome / Subsonic data.
 * Each hook reads credentials from the active NdServer (navidrome-store).
 */
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  ndGetAlbums,
  ndGetArtists,
  ndGetAlbum,
  ndGetArtist,
  ndGetSongs,
  ndGetPlaylists,
  ndGetPlaylist,
  ndGetStarred,
  ndStar,
  ndUnstar,
  type NdAlbum,
  type NdArtist,
  type NdPlaylist,
  type NdSong,
} from "@/lib/navidrome-client";
import { useNdActiveServer } from "@/lib/navidrome-store";

// ── Query key factory ────────────────────────────────────────────────────────

export const ndqk = {
  albums: (serverId: string) => ["nd", serverId, "albums"] as const,
  artists: (serverId: string) => ["nd", serverId, "artists"] as const,
  songs: (serverId: string) => ["nd", serverId, "songs"] as const,
  playlists: (serverId: string) => ["nd", serverId, "playlists"] as const,
  starred: (serverId: string) => ["nd", serverId, "starred"] as const,
  album: (serverId: string, albumId: string) =>
    ["nd", serverId, "album", albumId] as const,
  artist: (serverId: string, artistId: string) =>
    ["nd", serverId, "artist", artistId] as const,
  playlist: (serverId: string, playlistId: string) =>
    ["nd", serverId, "playlist", playlistId] as const,
};

const STALE = 5 * 60 * 1000;

// ── Top-level list hooks ─────────────────────────────────────────────────────

export function useNdAlbums() {
  const nd = useNdActiveServer();
  return useQuery<NdAlbum[]>({
    queryKey: nd ? ndqk.albums(nd.id) : ["nd", "disconnected", "albums"],
    queryFn: () => ndGetAlbums(nd!.baseUrl, nd!.user, nd!.password),
    enabled: !!nd,
    staleTime: STALE,
    placeholderData: [],
  });
}

export function useNdArtists() {
  const nd = useNdActiveServer();
  return useQuery<NdArtist[]>({
    queryKey: nd ? ndqk.artists(nd.id) : ["nd", "disconnected", "artists"],
    queryFn: () => ndGetArtists(nd!.baseUrl, nd!.user, nd!.password),
    enabled: !!nd,
    staleTime: STALE,
    placeholderData: [],
  });
}

export function useNdSongs() {
  const nd = useNdActiveServer();
  return useQuery<NdSong[]>({
    queryKey: nd ? ndqk.songs(nd.id) : ["nd", "disconnected", "songs"],
    queryFn: () => ndGetSongs(nd!.baseUrl, nd!.user, nd!.password),
    enabled: !!nd,
    staleTime: STALE,
    placeholderData: [],
  });
}

export function useNdPlaylists() {
  const nd = useNdActiveServer();
  return useQuery<NdPlaylist[]>({
    queryKey: nd ? ndqk.playlists(nd.id) : ["nd", "disconnected", "playlists"],
    queryFn: () => ndGetPlaylists(nd!.baseUrl, nd!.user, nd!.password),
    enabled: !!nd,
    staleTime: STALE,
    placeholderData: [],
  });
}

export function useNdStarred() {
  const nd = useNdActiveServer();
  return useQuery<NdSong[]>({
    queryKey: nd ? ndqk.starred(nd.id) : ["nd", "disconnected", "starred"],
    queryFn: () => ndGetStarred(nd!.baseUrl, nd!.user, nd!.password),
    enabled: !!nd,
    staleTime: STALE,
    placeholderData: [],
  });
}

// ── Detail hooks ─────────────────────────────────────────────────────────────

export function useNdAlbumDetail(albumId: string) {
  const nd = useNdActiveServer();
  return useQuery<{ album: NdAlbum; songs: NdSong[] } | null>({
    queryKey: nd ? ndqk.album(nd.id, albumId) : ["nd", "disconnected", "album", albumId],
    queryFn: () => ndGetAlbum(nd!.baseUrl, nd!.user, nd!.password, albumId),
    enabled: !!nd && !!albumId,
    staleTime: STALE,
  });
}

export function useNdArtistDetail(artistId: string) {
  const nd = useNdActiveServer();
  return useQuery<{ artist: NdArtist; albums: NdAlbum[] } | null>({
    queryKey: nd ? ndqk.artist(nd.id, artistId) : ["nd", "disconnected", "artist", artistId],
    queryFn: () => ndGetArtist(nd!.baseUrl, nd!.user, nd!.password, artistId),
    enabled: !!nd && !!artistId,
    staleTime: STALE,
  });
}

export function useNdPlaylistDetail(playlistId: string) {
  const nd = useNdActiveServer();
  return useQuery<{ playlist: NdPlaylist; songs: NdSong[] } | null>({
    queryKey: nd ? ndqk.playlist(nd.id, playlistId) : ["nd", "disconnected", "playlist", playlistId],
    queryFn: () => ndGetPlaylist(nd!.baseUrl, nd!.user, nd!.password, playlistId),
    enabled: !!nd && !!playlistId,
    staleTime: STALE,
  });
}

// ── Star / unstar mutations ───────────────────────────────────────────────────

export function useNdStar() {
  const nd = useNdActiveServer();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (songId: string) => {
      if (!nd) return Promise.resolve(false);
      return ndStar(nd.baseUrl, nd.user, nd.password, songId);
    },
    onSuccess: (_result, songId) => {
      if (!nd) return;
      // Optimistically add to starred cache.
      qc.setQueryData<NdSong[]>(ndqk.starred(nd.id), (prev = []) => {
        const exists = prev.some((s) => s.id === songId);
        if (exists) return prev;
        const allSongs = qc.getQueryData<NdSong[]>(ndqk.songs(nd.id)) ?? [];
        const song = allSongs.find((s) => s.id === songId);
        return song ? [song, ...prev] : prev;
      });
    },
  });
}

export function useNdUnstar() {
  const nd = useNdActiveServer();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (songId: string) => {
      if (!nd) return Promise.resolve(false);
      return ndUnstar(nd.baseUrl, nd.user, nd.password, songId);
    },
    onSuccess: (_result, songId) => {
      if (!nd) return;
      qc.setQueryData<NdSong[]>(ndqk.starred(nd.id), (prev = []) =>
        prev.filter((s) => s.id !== songId),
      );
    },
  });
}

// ── Starred ID set (for heart buttons) ───────────────────────────────────────

export function useNdStarredIds(): Set<string> {
  const { data = [] } = useNdStarred();
  return new Set(data.map((s) => s.id));
}
