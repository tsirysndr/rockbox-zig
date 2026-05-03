import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef } from "react";

import { qk } from "@/lib/queries";
import {
  RockboxClient,
  type DiscoveredService,
} from "@/lib/rockbox-client";
import {
  autoSelectFromDiscovery,
  hydrateSelectedServer,
} from "@/lib/server-store";

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

    unsubs.push(
      RockboxClient.subscribeStatus((s) => {
        qc.setQueryData(qk.status(), s);
      }),
    );
    unsubs.push(
      RockboxClient.subscribeCurrentTrack((t) => {
        qc.setQueryData(qk.currentTrack(), t);
      }),
    );
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
    };
  }, [qc, startDiscovery]);

  return null;
}
