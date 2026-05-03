import { Platform } from "react-native";

import { coverUrl } from "@/lib/cover-url";
import { RockboxClient } from "@/lib/rockbox-client";
import { useSelectedServer } from "@/lib/server-store";
import {
  RockboxNowPlaying,
  type NowPlayingMetadata,
} from "rockbox-now-playing";

import type { TrackSnapshot } from "rockbox-rpc";

/**
 * Translate a [TrackSnapshot] from the rockboxd stream into the metadata
 * shape consumed by the native media-session bridge. Resolves the album-art
 * URL through `coverUrl` so the path matches what `expo-image` uses in the
 * in-app player — the daemon serves covers on the GraphQL port at
 * `/covers/{id}`, not the raw HTTP port.
 */
export function metadataFor(
  track: TrackSnapshot,
  _server: ReturnType<typeof useSelectedServer>,
): NowPlayingMetadata {
  let artworkUrl: string | null = null;
  if (track.album_art) {
    if (/^https?:\/\//i.test(track.album_art)) {
      artworkUrl = track.album_art;
    } else {
      artworkUrl = coverUrl(track.album_art);
    }
  }
  return {
    trackId: track.id,
    title: track.title,
    artist: track.artist,
    album: track.album,
    artworkUrl,
    durationMs: track.duration_ms,
  };
}

/** True only when the native bridge can be used. */
export function nowPlayingEnabled(): boolean {
  return Platform.OS === "android" && RockboxNowPlaying.isAvailable;
}

/** Fire the matching transport RPC for a button tap. Errors are swallowed —
 *  if the daemon is offline the user already sees the connection state, and
 *  silently ignoring the tap is preferable to crashing the service. */
export async function dispatchAction(
  action: string,
  positionMs?: number,
): Promise<void> {
  if (!RockboxClient.isAvailable) return;
  try {
    switch (action) {
      case "play":
        await RockboxClient.play();
        break;
      case "pause":
        await RockboxClient.pause();
        break;
      case "playPause":
        await RockboxClient.playPause();
        break;
      case "next":
        await RockboxClient.next();
        break;
      case "prev":
        await RockboxClient.prev();
        break;
      case "stop":
        await RockboxClient.pause();
        break;
      case "seek":
        if (typeof positionMs === "number") {
          await RockboxClient.seek(positionMs);
        }
        break;
    }
  } catch {
    // ignored — see fn doc.
  }
}
