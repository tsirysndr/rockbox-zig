/**
 * TanStack Query hooks wrapping the rockbox-rpc native module.
 *
 * Pattern:
 * - Reads → `useQuery`. Server-streaming RPCs (`subscribe*`) populate the same
 *   cache entries via `queryClient.setQueryData`, see `RockboxStreams`.
 * - Writes → `useMutation`. After a write, invalidate the touched key so the
 *   reader refetches (the stream will usually beat the refetch and update the
 *   cache before it lands).
 *
 * Every hook short-circuits when `RockboxClient.isAvailable` is false — they
 * stay disabled, so the rest of the app can render the mock data without the
 * native module attached.
 */
import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationOptions,
  type UseQueryOptions,
} from "@tanstack/react-query";

import {
  RockboxClient,
  type DiscoveredService,
  type StatusSnapshot,
  type TrackSnapshot,
} from "@/lib/rockbox-client";

// ── Query keys ──────────────────────────────────────────────────────────────

export const qk = {
  all: ["rockbox"] as const,
  status: () => [...qk.all, "status"] as const,
  currentTrack: () => [...qk.all, "currentTrack"] as const,
  playlist: () => [...qk.all, "playlist"] as const,

  tracks: () => [...qk.all, "tracks"] as const,
  artists: () => [...qk.all, "artists"] as const,
  album: (id: string) => [...qk.all, "album", id] as const,
  liked: () => [...qk.all, "liked"] as const,
  search: (term: string) => [...qk.all, "search", term] as const,

  globalSettings: () => [...qk.all, "globalSettings"] as const,
  globalStatus: () => [...qk.all, "globalStatus"] as const,
  soundCurrent: (setting: number) =>
    [...qk.all, "soundCurrent", setting] as const,

  treeEntries: (path: string | null) => [...qk.all, "tree", path] as const,

  savedPlaylists: () => [...qk.all, "savedPlaylists"] as const,
  savedPlaylistTracks: (id: string) =>
    [...qk.all, "savedPlaylist", id, "tracks"] as const,
  smartPlaylists: () => [...qk.all, "smartPlaylists"] as const,
  smartPlaylistTracks: (id: string) =>
    [...qk.all, "smartPlaylist", id, "tracks"] as const,

  bluetoothAvailable: () => [...qk.all, "bluetoothAvailable"] as const,
  bluetoothDevices: () => [...qk.all, "bluetoothDevices"] as const,

  /** Cast / AirPlay output devices (HTTP REST). */
  outputDevices: () => [...qk.all, "outputDevices"] as const,

  discovered: () => [...qk.all, "discoveredServers"] as const,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

const enabledByDefault = () => RockboxClient.isAvailable;

type ROpts<T> = Omit<UseQueryOptions<T>, "queryKey" | "queryFn">;
type MOpts<TData, TVars> = UseMutationOptions<TData, Error, TVars>;

function rq<T>(
  key: readonly unknown[],
  fn: () => Promise<T>,
  opts?: ROpts<T>,
) {
  return useQuery<T>({
    queryKey: key,
    queryFn: fn,
    enabled: enabledByDefault(),
    staleTime: 5 * 60 * 1000,
    ...opts,
  });
}

// ── Reads ───────────────────────────────────────────────────────────────────

// 2s polling acts as a safety net for the streams. Stream events still
// drive immediate updates via `setQueryData`; the poll only kicks in if the
// stream task dies (e.g. transport blip) so the UI doesn't go stale.
const LIVE_REFETCH_MS = 2000;

export function useStatus<T = StatusSnapshot>(opts?: ROpts<T>) {
  return rq(qk.status(), () => RockboxClient.status() as Promise<T>, {
    refetchInterval: LIVE_REFETCH_MS,
    staleTime: 0,
    ...opts,
  });
}

export function useCurrentTrack<T = TrackSnapshot>(opts?: ROpts<T>) {
  return rq(
    qk.currentTrack(),
    () => RockboxClient.currentTrack() as Promise<T>,
    {
      refetchInterval: LIVE_REFETCH_MS,
      staleTime: 0,
      ...opts,
    },
  );
}

export function usePlaylistCurrent<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.playlist(),
    () => RockboxClient.getPlaylistCurrent() as Promise<T>,
    {
      refetchInterval: LIVE_REFETCH_MS,
      staleTime: 0,
      ...opts,
    },
  );
}

export function useTracks<T = unknown>(opts?: ROpts<T>) {
  return rq(qk.tracks(), () => RockboxClient.getTracks() as Promise<T>, opts);
}

export function useArtists<T = unknown>(opts?: ROpts<T>) {
  return rq(qk.artists(), () => RockboxClient.getArtists() as Promise<T>, opts);
}

export function useAlbums<T = unknown>(opts?: ROpts<T>) {
  return rq(
    [...qk.all, "albums"],
    () => RockboxClient.getAlbums() as Promise<T>,
    opts,
  );
}

export function useLikedAlbums<T = unknown>(opts?: ROpts<T>) {
  return rq(
    [...qk.all, "likedAlbums"],
    () => RockboxClient.getLikedAlbums() as Promise<T>,
    opts,
  );
}

export function useArtist<T = unknown>(id: string, opts?: ROpts<T>) {
  return rq(
    [...qk.all, "artist", id],
    () => RockboxClient.getArtist(id) as Promise<T>,
    { enabled: enabledByDefault() && id.length > 0, ...opts },
  );
}

export function useAlbum<T = unknown>(id: string, opts?: ROpts<T>) {
  return rq(qk.album(id), () => RockboxClient.getAlbum(id) as Promise<T>, {
    enabled: enabledByDefault() && id.length > 0,
    ...opts,
  });
}

export function useLikedTracks<T = unknown>(opts?: ROpts<T>) {
  return rq(qk.liked(), () => RockboxClient.getLikedTracks() as Promise<T>, opts);
}

export function useSearch<T = unknown>(term: string, opts?: ROpts<T>) {
  const t = term.trim();
  return rq(qk.search(t), () => RockboxClient.search(t) as Promise<T>, {
    enabled: enabledByDefault() && t.length > 0,
    staleTime: 30_000,
    ...opts,
  });
}

export function useGlobalSettings<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.globalSettings(),
    () => RockboxClient.getGlobalSettings() as Promise<T>,
    opts,
  );
}

export function useGlobalStatus<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.globalStatus(),
    () => RockboxClient.getGlobalStatus() as Promise<T>,
    opts,
  );
}

export function useSoundCurrent<T = unknown>(setting: number, opts?: ROpts<T>) {
  return rq(
    qk.soundCurrent(setting),
    () => RockboxClient.soundCurrent(setting) as Promise<T>,
    opts,
  );
}

export function useTreeEntries<T = unknown>(
  path: string | null,
  opts?: ROpts<T>,
) {
  return rq(
    qk.treeEntries(path),
    () => RockboxClient.treeGetEntries(path) as Promise<T>,
    opts,
  );
}

export function useSavedPlaylists<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.savedPlaylists(),
    () => RockboxClient.getSavedPlaylists() as Promise<T>,
    opts,
  );
}

export function useSavedPlaylistTracks<T = unknown>(
  id: string,
  opts?: ROpts<T>,
) {
  return rq(
    qk.savedPlaylistTracks(id),
    () => RockboxClient.getSavedPlaylistTracks(id) as Promise<T>,
    { enabled: enabledByDefault() && id.length > 0, ...opts },
  );
}

export function useSmartPlaylists<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.smartPlaylists(),
    () => RockboxClient.getSmartPlaylists() as Promise<T>,
    opts,
  );
}

export function useSmartPlaylistTracks<T = unknown>(
  id: string,
  opts?: ROpts<T>,
) {
  return rq(
    qk.smartPlaylistTracks(id),
    () => RockboxClient.getSmartPlaylistTracks(id) as Promise<T>,
    { enabled: enabledByDefault() && id.length > 0, ...opts },
  );
}

export function useBluetoothAvailable(opts?: ROpts<boolean>) {
  return rq(
    qk.bluetoothAvailable(),
    () => RockboxClient.bluetoothAvailable(),
    opts,
  );
}

export function useBluetoothDevices<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.bluetoothDevices(),
    () => RockboxClient.getBluetoothDevices() as Promise<T>,
    opts,
  );
}

export function useOutputDevices<T = unknown>(opts?: ROpts<T>) {
  return rq(
    qk.outputDevices(),
    () => RockboxClient.getDevices() as Promise<T>,
    {
      // Devices change rarely, but the picker should feel snappy.
      staleTime: 30_000,
      ...opts,
    },
  );
}

/**
 * Discovered LAN servers — populated by the discovery stream rather than a
 * one-shot fetch. The hook just reads the cache (defaulting to `[]`).
 */
export function useDiscoveredServers() {
  return useQuery<DiscoveredService[]>({
    queryKey: qk.discovered(),
    queryFn: () => Promise.resolve([] as DiscoveredService[]),
    enabled: enabledByDefault(),
    initialData: [],
    staleTime: Infinity,
  });
}

// ── Mutations ───────────────────────────────────────────────────────────────

const invalidate = (
  qc: ReturnType<typeof useQueryClient>,
  ...keys: readonly (readonly unknown[])[]
) => {
  for (const key of keys) qc.invalidateQueries({ queryKey: key as readonly unknown[] });
};

export function usePlayPause(opts?: MOpts<void, void>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => RockboxClient.playPause(),
    onSuccess: () => invalidate(qc, qk.status()),
    ...opts,
  });
}

export function useNext(opts?: MOpts<void, void>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => RockboxClient.next(),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist()),
    ...opts,
  });
}

export function usePrev(opts?: MOpts<void, void>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => RockboxClient.prev(),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist()),
    ...opts,
  });
}

export function useSeek(opts?: MOpts<void, number>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (positionMs: number) => RockboxClient.seek(positionMs),
    onSuccess: () => invalidate(qc, qk.currentTrack()),
    ...opts,
  });
}

export function usePlayAlbum(
  opts?: MOpts<void, { albumId: string; shuffle?: boolean }>,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ albumId, shuffle = false }) =>
      RockboxClient.playAlbum(albumId, shuffle),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function usePlayArtistTracks(
  opts?: MOpts<void, { artistId: string; shuffle?: boolean }>,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ artistId, shuffle = false }) =>
      RockboxClient.playArtistTracks(artistId, shuffle),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function usePlayTrack(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (path: string) => RockboxClient.playTrack(path),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function usePlayDirectory(
  opts?: MOpts<void, { path: string; shuffle?: boolean; position?: number }>,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ path, shuffle = false, position = -1 }) =>
      RockboxClient.playDirectory(path, shuffle, position),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function useJumpToQueuePosition(opts?: MOpts<void, number>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (pos: number) => RockboxClient.jumpToQueuePosition(pos),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function useRemoveFromQueue(opts?: MOpts<void, number>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (pos: number) => RockboxClient.removeFromQueue(pos),
    onSuccess: () => invalidate(qc, qk.playlist()),
    ...opts,
  });
}

export function useInsertTrackNext(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (path: string) => RockboxClient.insertTrackNext(path),
    onSuccess: () => invalidate(qc, qk.playlist()),
    ...opts,
  });
}

export function useInsertTrackLast(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (path: string) => RockboxClient.insertTrackLast(path),
    onSuccess: () => invalidate(qc, qk.playlist()),
    ...opts,
  });
}

export function useLikeTrack(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.likeTrack(id),
    onSuccess: () => invalidate(qc, qk.liked()),
    ...opts,
  });
}

export function useUnlikeTrack(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.unlikeTrack(id),
    onSuccess: () => invalidate(qc, qk.liked()),
    ...opts,
  });
}

export function useAdjustVolume(opts?: MOpts<void, number>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (steps: number) => RockboxClient.adjustVolume(steps),
    onSuccess: () => invalidate(qc, qk.soundCurrent(0)),
    ...opts,
  });
}

export function useSaveShuffle(opts?: MOpts<void, boolean>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (enabled: boolean) => RockboxClient.saveShuffle(enabled),
    onSuccess: () => invalidate(qc, qk.globalSettings(), qk.globalStatus()),
    ...opts,
  });
}

export function useSaveRepeat(opts?: MOpts<void, number>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (mode: number) => RockboxClient.saveRepeat(mode),
    onSuccess: () => invalidate(qc, qk.globalSettings(), qk.globalStatus()),
    ...opts,
  });
}

export function useCreateSavedPlaylist(
  opts?: MOpts<
    void,
    { name: string; description?: string | null; trackIds?: string[] }
  >,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ name, description = null, trackIds = [] }) =>
      RockboxClient.createSavedPlaylist(name, description, trackIds),
    onSuccess: () => invalidate(qc, qk.savedPlaylists()),
    ...opts,
  });
}

export function useDeleteSavedPlaylist(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.deleteSavedPlaylist(id),
    onSuccess: () => invalidate(qc, qk.savedPlaylists()),
    ...opts,
  });
}

export function useUpdateSavedPlaylist(
  opts?: MOpts<
    void,
    { id: string; name: string; description?: string | null }
  >,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name, description = null }) =>
      RockboxClient.updateSavedPlaylist(id, name, description),
    onSuccess: () => invalidate(qc, qk.savedPlaylists()),
    ...opts,
  });
}

export function useAddTrackToPlaylist(
  opts?: MOpts<void, { playlistId: string; trackId: string }>,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ playlistId, trackId }) =>
      RockboxClient.addTrackToPlaylist(playlistId, trackId),
    onSuccess: (_d, { playlistId }) =>
      invalidate(qc, qk.savedPlaylistTracks(playlistId)),
    ...opts,
  });
}

export function useRemoveTrackFromPlaylist(
  opts?: MOpts<void, { playlistId: string; trackId: string }>,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ playlistId, trackId }) =>
      RockboxClient.removeTrackFromPlaylist(playlistId, trackId),
    onSuccess: (_d, { playlistId }) =>
      invalidate(qc, qk.savedPlaylistTracks(playlistId)),
    ...opts,
  });
}

export function usePlaySavedPlaylist(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.playSavedPlaylist(id),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function usePlaySmartPlaylist(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.playSmartPlaylist(id),
    onSuccess: () => invalidate(qc, qk.currentTrack(), qk.playlist(), qk.status()),
    ...opts,
  });
}

export function useScanBluetooth(opts?: MOpts<void, void>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => RockboxClient.scanBluetooth(),
    onSuccess: () => invalidate(qc, qk.bluetoothDevices()),
    ...opts,
  });
}

export function useConnectBluetooth(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (address: string) => RockboxClient.connectBluetooth(address),
    onSuccess: () => invalidate(qc, qk.bluetoothDevices()),
    ...opts,
  });
}

export function useDisconnectBluetooth(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (address: string) => RockboxClient.disconnectBluetooth(address),
    onSuccess: () => invalidate(qc, qk.bluetoothDevices()),
    ...opts,
  });
}

export function useConnectDevice(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.connectDevice(id),
    onSuccess: () => invalidate(qc, qk.outputDevices()),
    ...opts,
  });
}

export function useDisconnectDevice(opts?: MOpts<void, string>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => RockboxClient.disconnectDevice(id),
    onSuccess: () => invalidate(qc, qk.outputDevices()),
    ...opts,
  });
}
