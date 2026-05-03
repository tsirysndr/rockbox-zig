/**
 * Returns the right bottom padding for a scrollable view so its last items
 * aren't hidden by the persistent miniplayer (or any docked UI).
 *
 * - Tab routes (`/`, `/search`, `/library`): React Navigation already insets
 *   content by the custom tab-bar height, so we just add a small breathing
 *   margin on top.
 * - Detail / settings routes when a track is loaded: reserve the floating
 *   miniplayer's height so the last row sits above it.
 * - Otherwise: a base safe-area-aware padding.
 */
import { usePathname } from "expo-router";
import { useSafeAreaInsets } from "react-native-safe-area-context";

import { FLOATING_MINIPLAYER_HEIGHT } from "@/components/persistent-mini-player";
import { usePlayer } from "@/lib/player-context";

const TAB_PATHS = new Set(["/", "/search", "/library"]);
const HIDE_MINIPLAYER_ON = new Set(["/player", "/playlist/new"]);

export function useBottomSpacing(extra = 8): number {
  const pathname = usePathname();
  const insets = useSafeAreaInsets();
  const { currentTrack } = usePlayer();

  // Tab routes: tab bar (with embedded miniplayer) is handled by RN
  // navigation's content inset; just leave a small visual margin.
  if (TAB_PATHS.has(pathname)) return extra + 16;

  const miniplayerVisible =
    !!currentTrack && !HIDE_MINIPLAYER_ON.has(pathname);
  const safeBottom = Math.max(insets.bottom, 8);

  if (miniplayerVisible) {
    return safeBottom + FLOATING_MINIPLAYER_HEIGHT + extra;
  }
  return safeBottom + extra;
}
