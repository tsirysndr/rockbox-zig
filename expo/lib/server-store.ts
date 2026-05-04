/**
 * Persistent selection of the rockboxd server to talk to.
 *
 * Stores host + port pieces separately so we can build both the gRPC URL
 * (host:grpcPort) and the GraphQL/covers URL (host:graphqlPort) without
 * re-parsing.
 *
 * In-memory subscribers get notified on every change, so React components
 * can re-render via `useSelectedServer()` without going through RQ.
 */
import AsyncStorage from "@react-native-async-storage/async-storage";
import { useEffect, useState } from "react";

import { RockboxClient, type DiscoveredService } from "@/lib/rockbox-client";

const STORAGE_KEY = "rockbox.selectedServer";
const DEFAULT_GRPC_PORT = 6061;
const DEFAULT_GRAPHQL_PORT = 6062;
const DEFAULT_HTTP_PORT = 6063;

export type ServerSelection = {
  /** Host or IP — no scheme, no port. */
  host: string;
  grpcPort: number;
  /** Used to build covers/* URLs. Defaults to 6062 when not discovered. */
  graphqlPort: number;
  /** Used by the cast/AirPlay HTTP endpoints. Defaults to 6063. */
  httpPort: number;
  /** Friendly name for UI. */
  label: string;
  /** Original mDNS fullname when discovered, else null for manual entries. */
  fullname: string | null;
};

let current: ServerSelection | null = null;
let hydrated = false;
const listeners = new Set<(s: ServerSelection | null) => void>();

function notify() {
  for (const l of listeners) l(current);
}

function applyToNative(s: ServerSelection | null) {
  if (!RockboxClient.isAvailable || !s) return;
  RockboxClient.setServerUrl(`http://${s.host}:${s.grpcPort}`);
  RockboxClient.setHttpUrl(`http://${s.host}:${s.httpPort}`);
}

async function persist(s: ServerSelection | null) {
  // Persistence is best-effort. If AsyncStorage's native module isn't
  // available (e.g. dev client built before the dep landed), skip silently
  // so the in-memory selection still works for the current session.
  try {
    if (s) await AsyncStorage.setItem(STORAGE_KEY, JSON.stringify(s));
    else await AsyncStorage.removeItem(STORAGE_KEY);
  } catch (err) {
    if (__DEV__) {
      console.warn(
        "[server-store] persistence skipped (AsyncStorage unavailable):",
        err,
      );
    }
  }
}

export async function hydrateSelectedServer(): Promise<ServerSelection | null> {
  if (hydrated) return current;
  try {
    const raw = await AsyncStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as ServerSelection;
      current = parsed;
      applyToNative(parsed);
    }
  } catch (err) {
    // Either AsyncStorage isn't loaded, or storage is corrupt — start clean.
    if (__DEV__) {
      console.warn(
        "[server-store] hydrate skipped (AsyncStorage unavailable):",
        err,
      );
    }
  }
  hydrated = true;
  notify();
  return current;
}

export function getSelectedServer(): ServerSelection | null {
  return current;
}

export async function setSelectedServer(
  s: ServerSelection | null,
): Promise<void> {
  current = s;
  applyToNative(s);
  await persist(s);
  notify();
}

/**
 * Update only the graphql/covers port for the active selection.
 * Useful when one mDNS event contributes the gRPC port and a sibling
 * event contributes the GraphQL port (rockboxd registers both).
 */
export async function setGraphqlPort(host: string, graphqlPort: number) {
  if (!current) return;
  if (current.host !== host) return;
  if (current.graphqlPort === graphqlPort) return;
  current = { ...current, graphqlPort };
  await persist(current);
  notify();
}

/** Mirror of `setGraphqlPort` for the HTTP / device-picker port. */
export async function setHttpPort(host: string, httpPort: number) {
  if (!current) return;
  if (current.host !== host) return;
  if (current.httpPort === httpPort) return;
  current = { ...current, httpPort };
  applyToNative(current);
  await persist(current);
  notify();
}

/** Returns the base URL for cover-art fetches (`/covers/{id}` is appended). */
export function coversBaseUrl(s: ServerSelection | null = current): string | null {
  if (!s) return null;
  return `http://${s.host}:${s.graphqlPort}/covers/`;
}

/**
 * Build a fully-qualified cover URL from a track's `album_art` id (which is
 * the suffix the rockbox library returns). Returns null if there's no server
 * yet, or if the id is empty.
 */
export function coverUrl(albumArtId: string | null | undefined): string | null {
  if (!albumArtId) return null;
  const base = coversBaseUrl();
  if (!base) return null;
  return `${base}${albumArtId}`;
}

/**
 * Pick a discovered service if nothing is selected yet. The rockbox-discovery
 * crate emits one event per service flavor (grpc-*, graphql-*, http-*, mpd-*)
 * sharing the same hostname; we prefer the gRPC service for the primary URL
 * and merge the graphql port in when the matching event arrives.
 */
export async function autoSelectFromDiscovery(
  svc: DiscoveredService,
): Promise<ServerSelection | null> {
  await hydrateSelectedServer();

  const host = preferredHost(svc);
  if (!host) return null;
  const flavor = serviceFlavor(svc);

  // Already have a selection? Just merge in extra port info if it's the same host.
  if (current) {
    if (flavor === "graphql" && current.host === host) {
      await setGraphqlPort(host, svc.port);
    } else if (flavor === "http" && current.host === host) {
      await setHttpPort(host, svc.port);
    }
    return current;
  }

  // No selection yet — only pick when we see a grpc-* event so we hit the
  // right port. Other flavors (http/mpd) get ignored until grpc lands.
  if (flavor !== "grpc") return null;

  const selection: ServerSelection = {
    host,
    grpcPort: svc.port,
    graphqlPort: DEFAULT_GRAPHQL_PORT,
    httpPort: DEFAULT_HTTP_PORT,
    label: pretty(svc.hostname) || host,
    fullname: svc.fullname,
  };
  await setSelectedServer(selection);
  return selection;
}

/**
 * Build a `ServerSelection` from a discovery event manually (used by the
 * server-picker UI when the user taps an entry).
 */
export function serverFromDiscovery(
  svc: DiscoveredService,
): ServerSelection | null {
  const host = preferredHost(svc);
  if (!host) return null;
  const flavor = serviceFlavor(svc);
  return {
    host,
    grpcPort: flavor === "grpc" ? svc.port : DEFAULT_GRPC_PORT,
    graphqlPort: flavor === "graphql" ? svc.port : DEFAULT_GRAPHQL_PORT,
    httpPort: flavor === "http" ? svc.port : DEFAULT_HTTP_PORT,
    label: pretty(svc.hostname) || host,
    fullname: svc.fullname,
  };
}

export function manualServer(host: string, port = DEFAULT_GRPC_PORT): ServerSelection {
  return {
    host,
    grpcPort: port,
    graphqlPort: DEFAULT_GRAPHQL_PORT,
    httpPort: DEFAULT_HTTP_PORT,
    label: host,
    fullname: null,
  };
}

// ── Helpers ─────────────────────────────────────────────────────────────────

function preferredHost(svc: DiscoveredService): string | null {
  // The bridge usually delivers an array, but be defensive against malformed
  // platform payloads — fall back to the hostname rather than throwing.
  const addresses: string[] = Array.isArray(svc.addresses) ? svc.addresses : [];

  // Drop addresses we can never connect to from the phone.
  const usable = addresses.filter((a) => {
    if (!a) return false;
    if (a === "0.0.0.0" || a === "::") return false;
    if (a.startsWith("127.")) return false; // loopback
    if (a.startsWith("169.254.")) return false; // link-local
    return true;
  });

  // Rank IPv4s: home LAN > corporate LAN > anything else > Docker bridges
  // (172.16-31.*, dished out by libmdns when rockboxd runs in Docker).
  // Reachable addresses come first; the 172.* range is kept as a last resort
  // in case it really is the LAN.
  const score = (a: string): number => {
    if (!/^\d+\.\d+\.\d+\.\d+$/.test(a)) return -1; // IPv6 etc., last
    if (a.startsWith("192.168.")) return 5;
    if (a.startsWith("10.")) return 4;
    if (isDockerBridge(a)) return 1;
    return 3;
  };
  const ipv4Sorted = usable
    .filter((a) => /^\d+\.\d+\.\d+\.\d+$/.test(a))
    .sort((a, b) => score(b) - score(a));

  if (ipv4Sorted.length > 0) return ipv4Sorted[0];

  // No IPv4 left → fall back to anything usable, then the hostname.
  if (usable.length > 0) return usable[0];
  const host = (svc.hostname ?? "").replace(/\.$/, "");
  return host || null;
}

/** True for the Docker default bridge range 172.16.0.0/12 (172.16.* – 172.31.*). */
function isDockerBridge(ipv4: string): boolean {
  if (!ipv4.startsWith("172.")) return false;
  const second = parseInt(ipv4.split(".")[1] ?? "", 10);
  return Number.isFinite(second) && second >= 16 && second <= 31;
}

function pretty(hostname: string): string {
  return hostname.replace(/\.local\.?$/, "").replace(/\.$/, "");
}

/**
 * Inspect the service name to decide which port flavor this event represents.
 * The rockbox-discovery crate names them `grpc-<id>`, `graphql-<id>`, `http-<id>`,
 * `mpd-<id>`. Falls back to `"unknown"` for arbitrary service names.
 */
export function serviceFlavor(
  svc: DiscoveredService,
): "grpc" | "graphql" | "http" | "mpd" | "unknown" {
  const name = svc.fullname.toLowerCase();
  if (name.includes("grpc-")) return "grpc";
  if (name.includes("graphql-")) return "graphql";
  if (name.includes("http-")) return "http";
  if (name.includes("mpd-")) return "mpd";
  return "unknown";
}

/**
 * Re-push the current selection into the native layer (SERVER_URL / HTTP_URL).
 * Call this whenever the native module may have been re-created without a
 * corresponding selection change (e.g. after app foreground on iOS).
 */
export function reapplyServerUrl() {
  applyToNative(current);
}

/** Subscribe to selection changes; returns an unsubscribe function. */
export function subscribeSelectedServer(
  listener: (s: ServerSelection | null) => void,
): () => void {
  listeners.add(listener);
  // Fire once with the current value so subscribers don't miss the boot state.
  listener(current);
  return () => {
    listeners.delete(listener);
  };
}

/** Hook flavor — re-renders on every selection change. */
export function useSelectedServer(): ServerSelection | null {
  const [s, setS] = useState<ServerSelection | null>(current);
  useEffect(() => subscribeSelectedServer(setS), []);
  return s;
}
