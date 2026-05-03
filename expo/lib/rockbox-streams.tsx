import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef } from "react";

import {
  dispatchAction,
  metadataFor,
  nowPlayingEnabled,
} from "@/lib/now-playing-bridge";
import { qk } from "@/lib/queries";
import {
  RockboxClient,
  type DiscoveredService,
  type StatusSnapshot,
  type TrackSnapshot,
} from "@/lib/rockbox-client";
import {
  autoSelectFromDiscovery,
  hydrateSelectedServer,
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

  useEffect(() => {
    if (!RockboxClient.isAvailable) return;

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
      RockboxClient.subscribeLibrary((snapshot) => {
        qc.setQueryData([...qk.all, "libraryStream"], snapshot);
      }),
    );

    void hydrateSelectedServer();
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
      for (const u of unsubs) u();
      if (nowPlayingEnabled()) RockboxNowPlaying.clear();
    };
  }, [qc, startDiscovery]);

  return null;
}
