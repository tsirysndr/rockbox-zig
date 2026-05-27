/**
 * Invisible component that scrobbles the current Navidrome track once the
 * playback threshold is reached (position >= max(30s, 50% of duration)).
 * Mount it once inside the app tree, below PlayerProvider.
 */
import { useEffect, useRef } from "react";

import { ndScrobble } from "@/lib/navidrome-client";
import { useNdActiveServer } from "@/lib/navidrome-store";
import { usePlayer } from "@/lib/player-context";

function isNdStreamUrl(path: string | undefined): string | null {
  if (!path) return null;
  if (!path.startsWith("http")) return null;
  const match = path.match(/[?&]id=([^&]+)/);
  return match ? match[1] : null;
}

export function NdScrobble() {
  const nd = useNdActiveServer();
  const { currentTrack, position, isPlaying } = usePlayer();
  const scrobbledRef = useRef<string | null>(null);

  const songId = isNdStreamUrl(currentTrack?.path);
  const duration = currentTrack?.duration ?? 0;

  // Reset when track changes.
  useEffect(() => {
    if (scrobbledRef.current !== songId) {
      scrobbledRef.current = null;
    }
  }, [songId]);

  useEffect(() => {
    if (!nd || !songId || !isPlaying) return;
    if (scrobbledRef.current === songId) return;
    if (duration === 0) return;
    const threshold = Math.max(30, Math.floor(duration / 2));
    if (position < threshold) return;

    scrobbledRef.current = songId;
    ndScrobble(nd.baseUrl, nd.user, nd.password, songId).catch(() => {});
  }, [nd, songId, isPlaying, position, duration]);

  return null;
}
