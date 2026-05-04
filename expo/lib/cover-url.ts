/**
 * Build the full URL for an album art asset.
 *
 * Mirrors `gpui/src/server.rs:get_covers_base()` —
 * `http://{host}:{graphqlPort}/covers/{album_art_id}`. The `album_art` field
 * on a `Track` proto is just the suffix (an id like `xxxx.jpg`); the host
 * and port come from the active `ServerSelection`.
 *
 * Falls back to `null` when no server is selected — UI should treat that as
 * "no artwork available" and render the music-note placeholder.
 */
import { coverUrl as base, useSelectedServer } from "@/lib/server-store";

export const coverUrl = base;

/** Reactive variant — re-renders when the selected server changes. */
export function useCoverUrl(albumArtId: string | null | undefined): string | null {
  const server = useSelectedServer();
  if (!albumArtId || !server) return null;
  return `http://${server.host}:${server.graphqlPort}/covers/${albumArtId}`;
}
