/**
 * `useIsConnected()` — true when the native gRPC module is loaded AND we
 * have a server selection. Components / providers branch on this to decide
 * whether to fetch real data or fall back to mock state.
 */
import { RockboxClient } from "@/lib/rockbox-client";
import { type ServerSelection, useSelectedServer } from "@/lib/server-store";

export function useIsConnected(): boolean {
  const server = useSelectedServer();
  return RockboxClient.isAvailable && server !== null;
}

function isLocalServer(server: ServerSelection): boolean {
  const host = server.host.toLowerCase();
  // Catch loopback addresses
  if (host === "localhost" || host.startsWith("127.") || host === "::1") return true;
  // Catch the embedded Android daemon: mDNS advertises the device's LAN IP
  // but the hostname resolves to "localhost", so the label ends up as "localhost"
  if (server.label.toLowerCase() === "localhost") return true;
  return false;
}

/** True when connected to a server that is NOT running on this device. */
export function useIsRemoteServer(): boolean {
  const server = useSelectedServer();
  return RockboxClient.isAvailable && server !== null && !isLocalServer(server);
}
