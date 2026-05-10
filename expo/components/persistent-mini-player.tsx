import { usePathname } from "expo-router";
import { View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";

import { MiniPlayer } from "@/components/mini-player";
import { usePlayer } from "@/lib/player-context";

/**
 * Floating miniplayer rendered at the root, used on screens that don't have
 * the bottom tab dock (album / artist / playlist / settings / etc.). On tab
 * routes the miniplayer lives inside the dock — so this component bows out
 * to avoid stacking two of them.
 *
 * Hidden entirely on `/player`, `/queue`, and `/playlist/new` since those
 * either own the playback UI directly or are full-screen modals.
 */
const HIDE_ON = new Set(["/player", "/playlist/new"]);
const TAB_PATHS = new Set(["/", "/search", "/library", "/files"]);

/** Approximate height the floating bar reserves at the bottom of the screen. */
export const FLOATING_MINIPLAYER_HEIGHT = 72;

export function PersistentMiniPlayer() {
  const pathname = usePathname();
  const insets = useSafeAreaInsets();
  const { currentTrack } = usePlayer();

  if (!currentTrack) return null;
  if (HIDE_ON.has(pathname)) return null;
  if (TAB_PATHS.has(pathname)) return null;

  const bottom = Math.max(insets.bottom, 8) + 4;

  return (
    <View
      pointerEvents="box-none"
      className="absolute left-2 right-2"
      style={{ bottom }}
    >
      <View
        className="bg-bg-dock rounded-xl overflow-hidden"
        style={{
          shadowColor: "#000",
          shadowOpacity: 0.5,
          shadowRadius: 16,
          shadowOffset: { width: 0, height: -4 },
        }}
      >
        <MiniPlayer />
      </View>
    </View>
  );
}
