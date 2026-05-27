/**
 * Persistent list of Navidrome / Subsonic servers.
 * Pattern mirrors server-store.ts: in-memory state + AsyncStorage + listener notify.
 */
import AsyncStorage from "@react-native-async-storage/async-storage";
import { useEffect, useState } from "react";

const STORAGE_KEY = "rockbox.navidromeServers";
const ACTIVE_KEY = "rockbox.navidromeActiveId";

export type NdServer = {
  id: string;
  label: string;
  baseUrl: string;
  user: string;
  /** Plain-text password — stored locally only, never sent to remote. */
  password: string;
};

let servers: NdServer[] = [];
let activeId: string | null = null;
let hydrated = false;

const listeners = new Set<() => void>();

function notify() {
  for (const l of listeners) l();
}

// ── Persistence ──────────────────────────────────────────────────────────────

async function save() {
  try {
    await AsyncStorage.setItem(STORAGE_KEY, JSON.stringify(servers));
    if (activeId) await AsyncStorage.setItem(ACTIVE_KEY, activeId);
    else await AsyncStorage.removeItem(ACTIVE_KEY);
  } catch {}
}

export async function hydrateNdServers(): Promise<void> {
  if (hydrated) return;
  hydrated = true;
  try {
    const raw = await AsyncStorage.getItem(STORAGE_KEY);
    if (raw) servers = JSON.parse(raw) as NdServer[];
    const aid = await AsyncStorage.getItem(ACTIVE_KEY);
    if (aid && servers.some((s) => s.id === aid)) activeId = aid;
    else if (servers.length > 0) activeId = servers[0].id;
  } catch {}
  notify();
}

// ── Mutations ────────────────────────────────────────────────────────────────

export async function ndAddServer(server: Omit<NdServer, "id">): Promise<NdServer> {
  const id = `nd-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  const s: NdServer = { ...server, id };
  servers = [...servers, s];
  if (!activeId) activeId = id;
  await save();
  notify();
  return s;
}

export async function ndRemoveServer(id: string): Promise<void> {
  servers = servers.filter((s) => s.id !== id);
  if (activeId === id) activeId = servers[0]?.id ?? null;
  await save();
  notify();
}

export async function ndSetActiveServer(id: string): Promise<void> {
  if (!servers.some((s) => s.id === id)) return;
  activeId = id;
  await save();
  notify();
}

// ── Reads ────────────────────────────────────────────────────────────────────

export function getNdServers(): NdServer[] { return servers; }
export function getNdActiveId(): string | null { return activeId; }
export function getNdActiveServer(): NdServer | null {
  return servers.find((s) => s.id === activeId) ?? null;
}

// ── Subscription / hook ───────────────────────────────────────────────────────

export function subscribeNdServers(cb: () => void): () => void {
  listeners.add(cb);
  cb();
  return () => { listeners.delete(cb); };
}

export function useNdServers(): { servers: NdServer[]; activeId: string | null } {
  const [state, setState] = useState({ servers, activeId });
  useEffect(() =>
    subscribeNdServers(() => setState({ servers: getNdServers(), activeId: getNdActiveId() })),
  []);
  return state;
}

export function useNdActiveServer(): NdServer | null {
  const [active, setActive] = useState<NdServer | null>(getNdActiveServer());
  useEffect(() =>
    subscribeNdServers(() => setActive(getNdActiveServer())),
  []);
  return active;
}
