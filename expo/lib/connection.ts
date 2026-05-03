/**
 * `useIsConnected()` — true when the native gRPC module is loaded AND we
 * have a server selection. Components / providers branch on this to decide
 * whether to fetch real data or fall back to mock state.
 */
import { RockboxClient } from "@/lib/rockbox-client";
import { useSelectedServer } from "@/lib/server-store";

export function useIsConnected(): boolean {
  const server = useSelectedServer();
  return RockboxClient.isAvailable && server !== null;
}
