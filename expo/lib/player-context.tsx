/**
 * Unified player context — feeds the mini-player / player / queue UI from
 * either the real gRPC streams (when a server is selected) or the bundled
 * mock state. The `usePlayer()` consumer surface is identical across modes.
 *
 * Real-mode wiring:
 * - `currentTrack` ← `useCurrentTrack` query (kept fresh by the
 *   `rockbox.currentTrack` event in `RockboxStreams`).
 * - `queue` ← `usePlaylistCurrent` (refreshed by `rockbox.playlist`).
 * - `isPlaying` ← `useStatus` (`status === 1`).
 * - `position` ← snapped to `currentTrack.elapsed_ms` whenever a new server
 *   event arrives, then interpolated locally at 1 Hz while playing so the
 *   seek bar moves smoothly between updates.
 * - `shuffle` / `repeat` ← `useGlobalSettings`.
 * - `liked` ← `useLikedTracks`.
 * - All actions (`play`, `pause`, `next`, `seek`, `toggleLike`, `playAlbum`
 *   et al.) call `RockboxClient.*` mutations.
 */
import { useQueryClient } from "@tanstack/react-query";
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { Alert } from "react-native";

import { useIsConnected } from "@/lib/connection";
import {
  qk,
  useCurrentTrack,
  useGlobalSettings,
  useLikedTracks,
  usePlaylistCurrent,
  useStatus,
} from "./queries";
import { trackFromProto } from "./proto-mappers";
import { RockboxClient } from "./rockbox-client";
import type { Playlist, RepeatMode, Track } from "./types";

type PlayerState = {
  queue: Track[];
  currentIdx: number;
  position: number;
  isPlaying: boolean;
  shuffle: boolean;
  repeat: RepeatMode;
  volume: number;
  liked: Set<string>;
  currentTrack: Track | undefined;
};

type UserPlaylistInput = {
  name: string;
  description?: string;
  isSmart?: boolean;
  rules?: string;
};

type PlayerActions = {
  play: () => void;
  pause: () => void;
  toggle: () => void;
  next: () => void;
  prev: () => void;
  seek: (secs: number) => void;
  setVolume: (vol: number) => void;
  toggleShuffle: () => void;
  cycleRepeat: () => void;
  toggleLike: (trackId: string) => void;
  jumpTo: (idx: number) => void;
  removeFromQueue: (idx: number) => void;
  clearQueue: () => void;
  playTrack: (track: Track) => void;
  playQueue: (
    tracks: Track[],
    opts?: { startIdx?: number; shuffle?: boolean },
  ) => void;
  playNext: (track: Track) => void;
  playLast: (track: Track) => void;
  openContextMenu: (track: Track) => void;
  closeContextMenu: () => void;
  createPlaylist: (input: UserPlaylistInput) => Playlist;
};

type ContextState = {
  contextTrack: Track | null;
  userPlaylists: Playlist[];
};

type PlayerContextValue = PlayerState & ContextState & PlayerActions;

const PlayerContext = createContext<PlayerContextValue | null>(null);

// ── Provider ────────────────────────────────────────────────────────────────

export function PlayerProvider({ children }: { children: React.ReactNode }) {
  const isReal = useIsConnected();

  // Shared cross-mode state (lives outside the gRPC layer).
  const [contextTrack, setContextTrack] = useState<Track | null>(null);
  const [userPlaylists, setUserPlaylists] = useState<Playlist[]>([]);

  const openContextMenu = useCallback(
    (t: Track) => setContextTrack(t),
    [],
  );
  const closeContextMenu = useCallback(() => setContextTrack(null), []);

  const createPlaylist = useCallback((input: UserPlaylistInput) => {
    const id = `user-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e6).toString(36)}`;
    const seed = encodeURIComponent(input.name || id);
    const playlist: Playlist = {
      id,
      name: input.name.trim() || "New Playlist",
      description: input.description?.trim() || undefined,
      isSmart: input.isSmart === true,
      rules: input.rules?.trim() || undefined,
      trackCount: 0,
      artwork: `https://picsum.photos/seed/${seed}/600/600`,
    };
    if (isReal && !input.isSmart) {
      // Best-effort: also create on the server for non-smart playlists.
      RockboxClient.createSavedPlaylist(playlist.name, playlist.description ?? null, []).catch(
        () => {},
      );
    }
    setUserPlaylists((list) => [playlist, ...list]);
    return playlist;
  }, [isReal]);

  // Real-mode value drives the player when a server is selected; we still
  // call the hook every render with `enabled: false` when disconnected so
  // React's hook ordering stays stable.
  const realValue = useRealPlayer({
    enabled: isReal,
    contextTrack,
    userPlaylists,
    openContextMenu,
    closeContextMenu,
    createPlaylist,
  });
  const idleValue = useIdlePlayer({
    contextTrack,
    userPlaylists,
    openContextMenu,
    closeContextMenu,
    createPlaylist,
  });

  const value = isReal ? realValue : idleValue;

  return (
    <PlayerContext.Provider value={value}>{children}</PlayerContext.Provider>
  );
}

export function usePlayer(): PlayerContextValue {
  const ctx = useContext(PlayerContext);
  if (!ctx) throw new Error("usePlayer must be used within PlayerProvider");
  return ctx;
}

// ── Real-mode driver ────────────────────────────────────────────────────────

type SharedSlice = {
  enabled: boolean;
  contextTrack: Track | null;
  userPlaylists: Playlist[];
  openContextMenu: (t: Track) => void;
  closeContextMenu: () => void;
  createPlaylist: (input: UserPlaylistInput) => Playlist;
};

type ProtoCurrentTrack = {
  id?: string;
  path?: string;
  title?: string;
  artist?: string;
  artist_id?: string;
  album?: string;
  album_id?: string;
  album_art?: string | null;
  length?: number;
  // Streaming subscription pushes `duration_ms` / `elapsed_ms` (TrackSnapshot
  // shape from `lib/rockbox-client.ts`); the unary `currentTrack()` may use
  // either depending on what populated the cache. Tolerate both.
  duration_ms?: number;
  elapsed_ms?: number;
  elapsed?: number;
};

type ProtoStatus = { status?: number };

type ProtoQueue = {
  index?: number;
  tracks?: Array<{ id?: string; path?: string; title?: string; artist?: string; album?: string; album_art?: string | null; length?: number }>;
};

type ProtoSettings = {
  playlist_shuffle?: boolean;
  repeat_mode?: number;
};

function useRealPlayer(slice: SharedSlice): PlayerContextValue {
  const { enabled } = slice;
  const qc = useQueryClient();

  const trackQ = useCurrentTrack<ProtoCurrentTrack>({ enabled });
  const statusQ = useStatus<ProtoStatus>({ enabled });
  const queueQ = usePlaylistCurrent<ProtoQueue>({ enabled });
  const settingsQ = useGlobalSettings<ProtoSettings>({ enabled });
  const likedQ = useLikedTracks<{ tracks?: Array<{ id?: string }>; ids?: string[] }>({ enabled });

  // Position interpolation: snap to server's elapsed_ms whenever a new track
  // event lands, tick locally between events while playing.
  const elapsedSecsFromServer =
    Math.floor(((trackQ.data?.elapsed_ms ?? trackQ.data?.elapsed ?? 0)) / 1000);
  const durationSecs = Math.max(
    0,
    Math.floor(
      (trackQ.data?.duration_ms ??
        trackQ.data?.length ??
        0) / 1000,
    ),
  );
  const trackId = trackQ.data?.id ?? "";
  const isPlaying = statusQ.data?.status === 1;

  const [position, setPosition] = useState(0);
  const lastSyncRef = useRef<{ trackId: string; elapsed: number }>({
    trackId: "",
    elapsed: 0,
  });

  // Snap to server elapsed when track id OR elapsed_ms changes.
  useEffect(() => {
    if (!enabled) return;
    const last = lastSyncRef.current;
    if (
      last.trackId !== trackId ||
      Math.abs(last.elapsed - elapsedSecsFromServer) >= 1
    ) {
      lastSyncRef.current = { trackId, elapsed: elapsedSecsFromServer };
      setPosition(elapsedSecsFromServer);
    }
  }, [enabled, trackId, elapsedSecsFromServer]);

  // 1 Hz tick while playing.
  useEffect(() => {
    if (!enabled || !isPlaying) return;
    const id = setInterval(() => {
      setPosition((p) => {
        if (durationSecs > 0 && p + 1 > durationSecs) return durationSecs;
        return p + 1;
      });
    }, 1000);
    return () => clearInterval(id);
  }, [enabled, isPlaying, durationSecs]);

  // Map proto data → app Track / Track[]
  const currentTrack: Track | undefined = useMemo(() => {
    if (!trackQ.data) return undefined;
    const t = trackFromProto({
      id: trackQ.data.id,
      path: trackQ.data.path,
      title: trackQ.data.title,
      artist: trackQ.data.artist,
      artist_id: trackQ.data.artist_id,
      album: trackQ.data.album,
      album_id: trackQ.data.album_id,
      album_art: trackQ.data.album_art ?? undefined,
      length: trackQ.data.length ?? trackQ.data.duration_ms ?? 0,
    });
    return t.id || t.title ? t : undefined;
  }, [trackQ.data]);

  const queue: Track[] = useMemo(() => {
    return (queueQ.data?.tracks ?? []).map((p) => trackFromProto(p));
  }, [queueQ.data]);
  const currentIdx = Math.max(0, queueQ.data?.index ?? 0);

  // Settings (shuffle / repeat)
  const shuffle = settingsQ.data?.playlist_shuffle === true;
  const repeat: RepeatMode = (() => {
    switch (settingsQ.data?.repeat_mode ?? 0) {
      case 1:
        return "all";
      case 2:
      case 3:
        return "one";
      default:
        return "off";
    }
  })();

  // Liked set
  const liked: Set<string> = useMemo(() => {
    const ids =
      likedQ.data?.ids ??
      (likedQ.data?.tracks ?? []).map((t) => t.id ?? "").filter(Boolean);
    return new Set(ids);
  }, [likedQ.data]);

  // ── Actions ──────────────────────────────────────────────────────────────

  const play = useCallback(() => {
    RockboxClient.play().catch(() => {});
  }, []);
  const pause = useCallback(() => {
    RockboxClient.pause().catch(() => {});
  }, []);
  const toggle = useCallback(() => {
    RockboxClient.playPause().catch(() => {});
  }, []);
  const next = useCallback(() => {
    RockboxClient.next().catch(() => {});
  }, []);
  const prev = useCallback(() => {
    RockboxClient.prev().catch(() => {});
  }, []);
  const seek = useCallback((secs: number) => {
    setPosition(secs); // optimistic UI
    RockboxClient.seek(Math.max(0, Math.floor(secs * 1000))).catch(() => {});
  }, []);
  const setVolumeAction = useCallback((_vol: number) => {
    // Volume is server-side adjusted in steps; the slider in the player
    // currently isn't wired, so just no-op here.
  }, []);
  const toggleShuffle = useCallback(() => {
    RockboxClient.saveShuffle(!shuffle).catch(() => {});
  }, [shuffle]);
  const cycleRepeat = useCallback(() => {
    // 0 off → 1 all → 0 (gpui only toggles between these two, mirror that).
    RockboxClient.saveRepeat(repeat === "off" ? 1 : 0).catch(() => {});
  }, [repeat]);
  const toggleLike = useCallback(
    (trackId: string) => {
      // Optimistic update — flip the cache immediately so the heart re-renders
      // before the server round-trips. We then refetch on success/error to
      // stay in sync with the daemon's authoritative state.
      const wasLiked = liked.has(trackId);
      qc.setQueryData<{ ids?: string[]; tracks?: { id?: string }[] }>(
        qk.liked(),
        (prev) => {
          const ids = prev?.ids
            ? [...prev.ids]
            : (prev?.tracks ?? []).map((t) => t.id ?? "").filter(Boolean);
          const next = wasLiked
            ? ids.filter((id) => id !== trackId)
            : [...ids, trackId];
          return { ids: next };
        },
      );
      const op = wasLiked
        ? RockboxClient.unlikeTrack(trackId)
        : RockboxClient.likeTrack(trackId);
      op.catch(() => {}).finally(() => {
        qc.invalidateQueries({ queryKey: qk.liked() });
      });
    },
    [liked, qc],
  );
  const jumpTo = useCallback((idx: number) => {
    RockboxClient.jumpToQueuePosition(idx).catch(() => {});
  }, []);
  const removeFromQueue = useCallback((idx: number) => {
    RockboxClient.removeFromQueue(idx).catch(() => {});
  }, []);

  // Remove all tracks after the current position, iterating from the end so
  // indices don't shift as items are deleted.
  const clearQueue = useCallback(() => {
    for (let i = queue.length - 1; i > currentIdx; i--) {
      RockboxClient.removeFromQueue(i).catch(() => {});
    }
  }, [queue.length, currentIdx]);

  const playTrack = useCallback(
    (track: Track) => {
      const doPlay = () => {
        if (track.path) RockboxClient.playTrack(track.path).catch(() => {});
      };
      if (queue.length > 0) {
        Alert.alert("Replace Queue", "This will clear the current queue.", [
          { text: "Cancel", style: "cancel" },
          { text: "Play", style: "destructive", onPress: doPlay },
        ]);
      } else {
        doPlay();
      }
    },
    [queue.length],
  );

  const playQueue = useCallback(
    (
      tracks: Track[],
      opts?: { startIdx?: number; shuffle?: boolean },
    ) => {
      const paths = tracks.map((t) => t.path).filter((p): p is string => !!p);
      if (paths.length === 0) return;
      const startIdx = opts?.startIdx ?? 0;
      const shouldShuffle = opts?.shuffle === true;
      const doPlay = () => {
        // Insert + start. Position 0 replaces queue head; rockbox's playlist
        // service handles the rest. After insert, jump to `startIdx`.
        RockboxClient.insertTracks(paths, 0, shouldShuffle)
          .then(() => RockboxClient.jumpToQueuePosition(startIdx))
          .catch(() => {});
      };
      if (queue.length > 0) {
        Alert.alert("Replace Queue", "This will clear the current queue.", [
          { text: "Cancel", style: "cancel" },
          { text: "Play", style: "destructive", onPress: doPlay },
        ]);
      } else {
        doPlay();
      }
    },
    [queue.length],
  );

  const playNext = useCallback((track: Track) => {
    if (track.path) RockboxClient.insertTrackNext(track.path).catch(() => {});
  }, []);
  const playLast = useCallback((track: Track) => {
    if (track.path) RockboxClient.insertTrackLast(track.path).catch(() => {});
  }, []);

  return {
    queue,
    currentIdx,
    position,
    isPlaying,
    shuffle,
    repeat,
    volume: 0.75,
    liked,
    currentTrack,

    contextTrack: slice.contextTrack,
    userPlaylists: slice.userPlaylists,

    play,
    pause,
    toggle,
    next,
    prev,
    seek,
    setVolume: setVolumeAction,
    toggleShuffle,
    cycleRepeat,
    toggleLike,
    jumpTo,
    removeFromQueue,
    clearQueue,
    playTrack,
    playQueue,
    playNext,
    playLast,
    openContextMenu: slice.openContextMenu,
    closeContextMenu: slice.closeContextMenu,
    createPlaylist: slice.createPlaylist,
  };
}

// ── Idle driver — used when no server is selected. Everything is a no-op
// and the UI is expected to render an "connect to a server" empty state.

type IdleSlice = Omit<SharedSlice, "enabled">;

function useIdlePlayer(slice: IdleSlice): PlayerContextValue {
  const noop = useCallback(() => {}, []);
  return {
    queue: [],
    currentIdx: 0,
    position: 0,
    isPlaying: false,
    shuffle: false,
    repeat: "off",
    volume: 0.75,
    liked: new Set<string>(),
    currentTrack: undefined,
    contextTrack: slice.contextTrack,
    userPlaylists: slice.userPlaylists,
    play: noop,
    pause: noop,
    toggle: noop,
    next: noop,
    prev: noop,
    seek: noop,
    setVolume: noop,
    toggleShuffle: noop,
    cycleRepeat: noop,
    toggleLike: noop,
    jumpTo: noop,
    removeFromQueue: noop,
    clearQueue: noop,
    playTrack: noop,
    playQueue: noop,
    playNext: noop,
    playLast: noop,
    openContextMenu: slice.openContextMenu,
    closeContextMenu: slice.closeContextMenu,
    createPlaylist: slice.createPlaylist,
  };
}

