import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { LIKED_TRACK_IDS, QUEUE } from "./mock-data";
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

export function PlayerProvider({ children }: { children: React.ReactNode }) {
  const [queue, setQueue] = useState<Track[]>(QUEUE);
  const [currentIdx, setCurrentIdx] = useState(0);
  const [position, setPosition] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [shuffle, setShuffle] = useState(false);
  const [repeat, setRepeat] = useState<RepeatMode>("off");
  const [volume, setVolume] = useState(0.75);
  const [liked, setLiked] = useState<Set<string>>(new Set(LIKED_TRACK_IDS));
  const [contextTrack, setContextTrack] = useState<Track | null>(null);
  const [userPlaylists, setUserPlaylists] = useState<Playlist[]>([]);

  const tickRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const currentTrack = queue[currentIdx];

  // Tick position once per second while playing.
  useEffect(() => {
    if (tickRef.current) {
      clearInterval(tickRef.current);
      tickRef.current = null;
    }
    if (isPlaying && currentTrack) {
      tickRef.current = setInterval(() => {
        setPosition((p) => {
          if (p + 1 >= currentTrack.duration) {
            // auto-advance
            setTimeout(() => {
              setCurrentIdx((idx) => {
                if (repeat === "one") return idx;
                if (idx + 1 < queue.length) return idx + 1;
                if (repeat === "all") return 0;
                return idx;
              });
              setPosition(0);
              if (repeat === "off" && currentIdx + 1 >= queue.length) {
                setIsPlaying(false);
              }
            }, 0);
            return 0;
          }
          return p + 1;
        });
      }, 1000);
    }
    return () => {
      if (tickRef.current) {
        clearInterval(tickRef.current);
        tickRef.current = null;
      }
    };
  }, [isPlaying, currentTrack, queue.length, repeat, currentIdx]);

  const play = useCallback(() => setIsPlaying(true), []);
  const pause = useCallback(() => setIsPlaying(false), []);
  const toggle = useCallback(() => setIsPlaying((p) => !p), []);

  const next = useCallback(() => {
    setCurrentIdx((idx) => (idx + 1 < queue.length ? idx + 1 : repeat === "all" ? 0 : idx));
    setPosition(0);
  }, [queue.length, repeat]);

  const prev = useCallback(() => {
    setPosition((p) => {
      if (p > 3) return 0;
      setCurrentIdx((idx) => Math.max(0, idx - 1));
      return 0;
    });
  }, []);

  const seek = useCallback((secs: number) => setPosition(Math.max(0, secs)), []);

  const toggleShuffle = useCallback(() => setShuffle((s) => !s), []);

  const cycleRepeat = useCallback(
    () =>
      setRepeat((r) => (r === "off" ? "all" : r === "all" ? "one" : "off")),
    [],
  );

  const toggleLike = useCallback(
    (trackId: string) =>
      setLiked((s) => {
        const next = new Set(s);
        if (next.has(trackId)) next.delete(trackId);
        else next.add(trackId);
        return next;
      }),
    [],
  );

  const jumpTo = useCallback((idx: number) => {
    setCurrentIdx(idx);
    setPosition(0);
    setIsPlaying(true);
  }, []);

  const removeFromQueue = useCallback(
    (idx: number) =>
      setQueue((q) => {
        const next = q.filter((_, i) => i !== idx);
        if (idx < currentIdx) setCurrentIdx((c) => c - 1);
        else if (idx === currentIdx) {
          setPosition(0);
          if (idx >= next.length) setCurrentIdx(Math.max(0, next.length - 1));
        }
        return next;
      }),
    [currentIdx],
  );

  const playTrack = useCallback(
    (track: Track) =>
      setQueue((q) => {
        const existing = q.findIndex((t) => t.id === track.id);
        if (existing >= 0) {
          setCurrentIdx(existing);
          setPosition(0);
          setIsPlaying(true);
          return q;
        }
        const next = [...q, track];
        setCurrentIdx(next.length - 1);
        setPosition(0);
        setIsPlaying(true);
        return next;
      }),
    [],
  );

  const playNext = useCallback(
    (track: Track) =>
      setQueue((q) => {
        const stripped = q.filter((t) => t.id !== track.id);
        const insertIdx = currentIdx + 1;
        const next = [
          ...stripped.slice(0, insertIdx),
          track,
          ...stripped.slice(insertIdx),
        ];
        return next;
      }),
    [currentIdx],
  );

  const playLast = useCallback(
    (track: Track) =>
      setQueue((q) => {
        if (q.find((t) => t.id === track.id)) return q;
        return [...q, track];
      }),
    [],
  );

  const openContextMenu = useCallback(
    (track: Track) => setContextTrack(track),
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
    setUserPlaylists((list) => [playlist, ...list]);
    return playlist;
  }, []);

  const playQueue = useCallback(
    (tracks: Track[], opts?: { startIdx?: number; shuffle?: boolean }) => {
      if (tracks.length === 0) return;
      let nextQueue = tracks;
      let startIdx = opts?.startIdx ?? 0;
      if (opts?.shuffle) {
        const shuffled = [...tracks];
        for (let i = shuffled.length - 1; i > 0; i--) {
          const j = Math.floor(Math.random() * (i + 1));
          [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
        }
        nextQueue = shuffled;
        startIdx = 0;
        setShuffle(true);
      }
      setQueue(nextQueue);
      setCurrentIdx(Math.max(0, Math.min(startIdx, nextQueue.length - 1)));
      setPosition(0);
      setIsPlaying(true);
    },
    [],
  );

  const value = useMemo<PlayerContextValue>(
    () => ({
      queue,
      currentIdx,
      position,
      isPlaying,
      shuffle,
      repeat,
      volume,
      liked,
      currentTrack,
      play,
      pause,
      toggle,
      next,
      prev,
      seek,
      setVolume,
      toggleShuffle,
      cycleRepeat,
      toggleLike,
      jumpTo,
      removeFromQueue,
      playTrack,
      playQueue,
      playNext,
      playLast,
      contextTrack,
      openContextMenu,
      closeContextMenu,
      userPlaylists,
      createPlaylist,
    }),
    [
      queue,
      currentIdx,
      position,
      isPlaying,
      shuffle,
      repeat,
      volume,
      liked,
      currentTrack,
      play,
      pause,
      toggle,
      next,
      prev,
      seek,
      toggleShuffle,
      cycleRepeat,
      toggleLike,
      jumpTo,
      removeFromQueue,
      playTrack,
      playQueue,
      playNext,
      playLast,
      contextTrack,
      openContextMenu,
      closeContextMenu,
      userPlaylists,
      createPlaylist,
    ],
  );

  return <PlayerContext.Provider value={value}>{children}</PlayerContext.Provider>;
}

export function usePlayer(): PlayerContextValue {
  const ctx = useContext(PlayerContext);
  if (!ctx) throw new Error("usePlayer must be used within PlayerProvider");
  return ctx;
}
