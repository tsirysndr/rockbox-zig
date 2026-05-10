import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import { AppState } from "react-native";

import {
  dispatchAction,
  metadataFor,
  nowPlayingEnabled,
} from "@/lib/now-playing-bridge";
import { qk } from "@/lib/queries";
import {
  RockboxClient,
  type DiscoveredService,
  type PlaylistSnapshot,
  type StatusSnapshot,
  type TrackSnapshot,
} from "@/lib/rockbox-client";
import {
  autoSelectFromDiscovery,
  coversBaseUrl,
  hydrateSelectedServer,
  reapplyServerUrl,
  useSelectedServer,
} from "@/lib/server-store";
import { RockboxNowPlaying } from "rockbox-now-playing";

/**
 * Mounts the rockbox streaming subscriptions and pipes their events into
 * the React Query cache. Render this once at the top of the app — the
 * streams stay live until unmount, the cache stays fresh in real time.
 */

// Module-level handle so non-React code (e.g. the server picker's
// pull-to-refresh) can ask the streams component to restart discovery.
let restartDiscoveryImpl: (() => void) | null = null;

export function restartDiscovery() {
  restartDiscoveryImpl?.();
}

export function RockboxStreams() {
  const qc = useQueryClient();
  const discoveryUnsubRef = useRef<(() => void) | null>(null);
  const server = useSelectedServer();
  const serverRef = useRef(server);
  serverRef.current = server;
  const lastTrackRef = useRef<TrackSnapshot | null>(null);
  const lastStatusRef = useRef<StatusSnapshot | null>(null);

  // Bumped on every app foreground so subscriptions restart after the native
  // module is re-created in background (iOS) or the process is resumed (Android).
  const [reconnectEpoch, setReconnectEpoch] = useState(0);

  // Re-apply the server URL to native and restart subscriptions whenever the
  // app comes to the foreground. On iOS the RockboxRpcModule instance may be
  // deallocated in background (weak-self poll loops exit); on Android the
  // Kotlin coroutine scope may be cancelled. Either way, re-subscribing on
  // active ensures fresh poll loops with the current server URL.
  useEffect(() => {
    if (!RockboxClient.isAvailable) return;
    const sub = AppState.addEventListener("change", (state) => {
      if (state === "active") {
        reapplyServerUrl();
        setReconnectEpoch((e) => e + 1);
      }
    });
    return () => sub.remove();
  }, []);

  // Identity of the active gRPC endpoint. Used as the dep that re-runs the
  // playback-stream subscriptions and clears the React Query cache when the
  // user switches servers in Settings.
  const serverUrlKey = server ? `${server.host}:${server.grpcPort}` : "";
  const previousServerUrlRef = useRef<string>(serverUrlKey);

  // Push the cover base URL into the now-playing service on every server
  // change so it can resolve `album_art` ids while JS is suspended.
  useEffect(() => {
    if (!nowPlayingEnabled()) return;
    const base = coversBaseUrl(server);
    if (base) RockboxNowPlaying.setCoverBaseUrl(base);
  }, [server]);

  // On every change of the active server, drop per-server cache entries so
  // the screens refetch from the new endpoint instead of showing the previous
  // server's data until staleTime expires. Skip caches that are populated by
  // LAN/HTTP discovery (not the selected server): mdns-sd only fires
  // `ServiceResolved` once per service, so dropping `discoveredServers`
  // would empty the server picker until the user pull-to-refreshes.
  useEffect(() => {
    if (previousServerUrlRef.current === serverUrlKey) return;
    previousServerUrlRef.current = serverUrlKey;
    qc.removeQueries({
      queryKey: qk.all,
      predicate: (q) => {
        const k = q.queryKey;
        if (Array.isArray(k) && k.length >= 2) {
          // discoveredServers   → mDNS browse (independent of selected server)
          // outputDevices       → cast/AirPlay HTTP picker (refetch on its own)
          if (k[1] === "discoveredServers" || k[1] === "outputDevices") {
            return false;
          }
        }
        return true;
      },
    });
  }, [qc, serverUrlKey]);

  const startDiscovery = useCallback(() => {
    // Tear down the previous browse, if any, so we get a fresh
    // ServiceDaemon and can re-resolve previously-cached entries.
    discoveryUnsubRef.current?.();
    discoveryUnsubRef.current = RockboxClient.subscribeDiscovery((svc) => {
      qc.setQueryData<DiscoveredService[]>(qk.discovered(), (prev) => {
        const list = prev ?? [];
        if (list.some((s) => s.fullname === svc.fullname)) return list;
        return [...list, svc];
      });
      void autoSelectFromDiscovery(svc);
    });
  }, [qc]);

  // ── Discovery + initial hydration (mount-only) ──────────────────────────
  useEffect(() => {
    if (!RockboxClient.isAvailable) return;
    // After hydration (including JS reload), invalidate all per-server query
    // data so screens refetch immediately rather than staying empty.
    void hydrateSelectedServer().then((server) => {
      if (server) qc.invalidateQueries({ queryKey: qk.all });
    });
    startDiscovery();
    // Expose the restart handle for callers outside React.
    // Note: we deliberately DO NOT clear the cache here — mdns-sd takes a
    // few seconds to re-resolve previously-seen services, so clearing first
    // would leave the picker empty even when servers are still online. The
    // dedup-by-fullname guard on each event keeps the list clean.
    restartDiscoveryImpl = () => {
      startDiscovery();
    };
    return () => {
      restartDiscoveryImpl = null;
      discoveryUnsubRef.current?.();
      discoveryUnsubRef.current = null;
    };
  }, [startDiscovery, qc]);

  // ── Playback streams (re-runs whenever the active server changes) ───────
  // The Rust-side `rb_subscribe_*` fns capture the server URL at spawn time
  // and never re-read it, so a server switch requires unsubscribing and
  // re-subscribing to point the new tonic stream at the new endpoint.
  useEffect(() => {
    if (!RockboxClient.isAvailable) return;
    // No server yet (e.g. right after JS reload, before AsyncStorage hydrates).
    // Skip — the effect re-runs once serverUrlKey becomes non-empty.
    if (!serverUrlKey) return;

    const unsubs: Array<() => void> = [];

    const pushNowPlaying = (
      track: TrackSnapshot | null,
      status: StatusSnapshot | null,
    ) => {
      if (!nowPlayingEnabled()) return;
      if (!track || !track.id) {
        RockboxNowPlaying.clear();
        return;
      }
      RockboxNowPlaying.update(metadataFor(track, serverRef.current), {
        isPlaying: status?.status === 1,
        positionMs: track.elapsed_ms,
      });
    };

    unsubs.push(
      RockboxClient.subscribeStatus((s) => {
        qc.setQueryData(qk.status(), s);
        lastStatusRef.current = s;
        if (lastTrackRef.current && nowPlayingEnabled()) {
          RockboxNowPlaying.setPlayback({
            isPlaying: s.status === 1,
            positionMs: lastTrackRef.current.elapsed_ms,
          });
        }
      }),
    );
    unsubs.push(
      RockboxClient.subscribeCurrentTrack((t) => {
        qc.setQueryData(qk.currentTrack(), t);
        lastTrackRef.current = t;
        pushNowPlaying(t, lastStatusRef.current);
      }),
    );
    if (nowPlayingEnabled()) {
      const off = RockboxNowPlaying.onAction((e) => {
        void dispatchAction(e.action, e.positionMs);
      });
      unsubs.push(off);

      // React to *any* cache change — covers the stream, the 2s polling
      // fallback, optimistic updates, etc. Saves us from chasing whichever
      // source happens to deliver the current track first.
      const trackKey = JSON.stringify(qk.currentTrack());
      const statusKey = JSON.stringify(qk.status());
      const sub = qc.getQueryCache().subscribe((event) => {
        const k = JSON.stringify(event.query.queryKey);
        if (k !== trackKey && k !== statusKey) return;
        const t = qc.getQueryData<TrackSnapshot>(qk.currentTrack()) ?? null;
        const s = qc.getQueryData<StatusSnapshot>(qk.status()) ?? null;
        if (t) lastTrackRef.current = t;
        if (s) lastStatusRef.current = s;
        pushNowPlaying(lastTrackRef.current, lastStatusRef.current);
      });
      unsubs.push(() => sub());

      // Hydrate from whatever's already in the cache.
      const cachedTrack = qc.getQueryData<TrackSnapshot>(qk.currentTrack());
      const cachedStatus = qc.getQueryData<StatusSnapshot>(qk.status());
      if (cachedTrack) {
        lastTrackRef.current = cachedTrack;
        if (cachedStatus) lastStatusRef.current = cachedStatus;
        pushNowPlaying(cachedTrack, cachedStatus ?? null);
      }
    }
    unsubs.push(
      RockboxClient.subscribePlaylist((p) => {
        qc.setQueryData(qk.playlist(), p);
      }),
    );
    unsubs.push(
      RockboxClient.subscribeLibrary(() => {
        // Scan finished — invalidate all library data so every screen
        // refetches tracks, artists, albums, playlists and genres.
        qc.invalidateQueries({ queryKey: qk.tracks() });
        qc.invalidateQueries({ queryKey: qk.artists() });
        qc.invalidateQueries({ queryKey: qk.albums() });
        qc.invalidateQueries({ queryKey: qk.likedAlbums() });
        qc.invalidateQueries({ queryKey: qk.genres() });
        qc.invalidateQueries({ queryKey: qk.savedPlaylists() });
        qc.invalidateQueries({ queryKey: qk.smartPlaylists() });
      }),
    );

    // Startup restore: StreamPlaylist (SimpleBroker) only delivers *future*
    // publishes, so if the firmware already loaded saved state before we
    // subscribed, we'd never receive the initial queue.  Retry
    // playlistResume + getPlaylistCurrent up to 10× (5 s) until the queue
    // is non-empty — the C firmware takes variable time to load saved state
    // after gRPC binds.
    let cancelled = false;
    (async () => {
      for (let attempt = 0; attempt < 10; attempt++) {
        if (cancelled) return;
        if (attempt > 0) {
          await new Promise((r) => setTimeout(r, 500));
          if (cancelled) return;
        }
        try {
          await RockboxClient.playlistResume();
          const snapshot = (await RockboxClient.getPlaylistCurrent()) as PlaylistSnapshot;
          if (snapshot?.tracks?.length) {
            qc.setQueryData(qk.playlist(), snapshot);
            return;
          }
        } catch {
          // server not ready yet — retry
        }
      }
    })();

    return () => {
      cancelled = true;
      for (const u of unsubs) u();
      if (nowPlayingEnabled()) RockboxNowPlaying.clear();
    };
    // serverUrlKey rebinds the effect to the active server.
    // reconnectEpoch forces a restart on every app foreground so poll loops
    // that exited (module re-created in background) are replaced with fresh ones.
  }, [qc, serverUrlKey, reconnectEpoch]);

  return null;
}
