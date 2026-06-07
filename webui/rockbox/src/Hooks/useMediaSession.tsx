// Wires the browser's Media Session API to the current track + control
// callbacks so OS-level notifications (lock screen / media keys / Bluetooth
// remotes / smartwatch) display metadata and drive playback.
//
// Only active while the HLS player is the local sink — the system notification
// is anchored to the in-page <audio> element that hls-audio.ts owns, and the
// title/artist/album/cover come from the upstream Rockbox now-playing state.
// When the user switches to a non-HLS device (built-in / FIFO / AirPlay / …)
// the browser isn't producing audio locally, so we tear the session down.
//
// Time units: `nowPlaying.duration` / `.progress` are in milliseconds (matches
// useTimeFormat); Media Session uses seconds — we convert both ways.

import { useEffect, useRef } from "react";
import { useHlsAudio } from "../lib/hls-audio";
import { CurrentTrack } from "../Types/track";

type MediaSessionCallbacks = {
  onPlay: () => void | Promise<void>;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onSeek: (positionMs: number) => void;
};

type Args = {
  nowPlaying?: CurrentTrack;
} & MediaSessionCallbacks;

const SEEK_STEP_SECONDS = 10;

const isMediaSessionSupported = () =>
  typeof navigator !== "undefined" && "mediaSession" in navigator;

const ARTWORK_SIZES = ["96x96", "192x192", "256x256", "384x384", "512x512"];

const buildArtwork = (cover?: string): MediaImage[] => {
  if (!cover) return [];
  return ARTWORK_SIZES.map((sizes) => ({ src: cover, sizes }));
};

const setHandler = (
  action: MediaSessionAction,
  handler: MediaSessionActionHandler | null,
) => {
  try {
    navigator.mediaSession.setActionHandler(action, handler);
  } catch {
    // Action unsupported by the UA — ignore.
  }
};

export const useMediaSession = ({
  nowPlaying,
  onPlay,
  onPause,
  onNext,
  onPrevious,
  onSeek,
}: Args) => {
  const { attached } = useHlsAudio();

  // Refs let the action handlers stay bound for the lifetime of the hook
  // while still reading the latest state/callbacks. Without this we would
  // rebind every render (cheap but noisy) and risk capturing stale closures.
  const nowPlayingRef = useRef<CurrentTrack | undefined>(nowPlaying);
  const callbacksRef = useRef<MediaSessionCallbacks>({
    onPlay,
    onPause,
    onNext,
    onPrevious,
    onSeek,
  });

  useEffect(() => {
    nowPlayingRef.current = nowPlaying;
  }, [nowPlaying]);

  useEffect(() => {
    callbacksRef.current = { onPlay, onPause, onNext, onPrevious, onSeek };
  }, [onPlay, onPause, onNext, onPrevious, onSeek]);

  // Install action handlers when HLS becomes active, tear them down when it
  // doesn't. Browsers only surface the system notification while there is an
  // <audio> element actually emitting sound, so this also avoids polluting
  // the OS media UI on devices that route audio elsewhere.
  useEffect(() => {
    if (!isMediaSessionSupported() || !attached) {
      if (isMediaSessionSupported()) {
        navigator.mediaSession.metadata = null;
        navigator.mediaSession.playbackState = "none";
      }
      return;
    }

    setHandler("play", () => {
      void callbacksRef.current.onPlay();
    });
    setHandler("pause", () => {
      callbacksRef.current.onPause();
    });
    setHandler("nexttrack", () => {
      callbacksRef.current.onNext();
    });
    setHandler("previoustrack", () => {
      callbacksRef.current.onPrevious();
    });
    setHandler("stop", () => {
      callbacksRef.current.onPause();
    });
    setHandler("seekto", (details) => {
      if (details.seekTime == null) return;
      callbacksRef.current.onSeek(Math.floor(details.seekTime * 1000));
    });
    setHandler("seekbackward", (details) => {
      const stepMs = Math.floor(
        (details.seekOffset ?? SEEK_STEP_SECONDS) * 1000,
      );
      const current = nowPlayingRef.current?.progress ?? 0;
      callbacksRef.current.onSeek(Math.max(0, current - stepMs));
    });
    setHandler("seekforward", (details) => {
      const stepMs = Math.floor(
        (details.seekOffset ?? SEEK_STEP_SECONDS) * 1000,
      );
      const current = nowPlayingRef.current?.progress ?? 0;
      const duration = nowPlayingRef.current?.duration ?? 0;
      const target = current + stepMs;
      callbacksRef.current.onSeek(
        duration > 0 ? Math.min(duration, target) : target,
      );
    });

    return () => {
      const actions: MediaSessionAction[] = [
        "play",
        "pause",
        "stop",
        "nexttrack",
        "previoustrack",
        "seekto",
        "seekbackward",
        "seekforward",
      ];
      actions.forEach((a) => setHandler(a, null));
      navigator.mediaSession.metadata = null;
      navigator.mediaSession.playbackState = "none";
      try {
        navigator.mediaSession.setPositionState();
      } catch {
        // Some UAs throw when called with no args — best-effort.
      }
    };
  }, [attached]);

  // Metadata: re-render only when the identifying fields change so we don't
  // recreate the MediaMetadata object every progress tick (cover fetches can
  // be expensive on the OS side).
  useEffect(() => {
    if (!isMediaSessionSupported() || !attached) return;
    if (!nowPlaying?.title) {
      navigator.mediaSession.metadata = null;
      return;
    }
    navigator.mediaSession.metadata = new MediaMetadata({
      title: nowPlaying.title ?? "",
      artist: nowPlaying.artist ?? "",
      album: nowPlaying.album ?? "",
      artwork: buildArtwork(nowPlaying.cover),
    });
  }, [
    attached,
    nowPlaying?.id,
    nowPlaying?.title,
    nowPlaying?.artist,
    nowPlaying?.album,
    nowPlaying?.cover,
  ]);

  // Playback state — drives the play/pause glyph in the OS UI.
  useEffect(() => {
    if (!isMediaSessionSupported() || !attached) return;
    navigator.mediaSession.playbackState = nowPlaying?.isPlaying
      ? "playing"
      : "paused";
  }, [attached, nowPlaying?.isPlaying]);

  // Position state — drives the scrubber. Browsers extrapolate position
  // forward using `playbackRate`, so we only need to call this when the
  // truth diverges (track change, seek, pause/resume).
  useEffect(() => {
    if (!isMediaSessionSupported() || !attached) return;
    const duration = (nowPlaying?.duration ?? 0) / 1000;
    if (!isFinite(duration) || duration <= 0) return;
    const rawPosition = (nowPlaying?.progress ?? 0) / 1000;
    const position = Math.max(0, Math.min(rawPosition, duration));
    try {
      navigator.mediaSession.setPositionState({
        duration,
        position,
        playbackRate: nowPlaying?.isPlaying ? 1.0 : 0.0,
      });
    } catch {
      // Position state can throw if duration/position are not finite — ignore.
    }
  }, [
    attached,
    nowPlaying?.id,
    nowPlaying?.duration,
    nowPlaying?.progress,
    nowPlaying?.isPlaying,
  ]);
};
