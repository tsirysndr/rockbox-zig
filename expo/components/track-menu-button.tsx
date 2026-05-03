import { Ionicons } from "@expo/vector-icons";
import { Pressable } from "react-native";

import { Colors } from "@/constants/theme";
import { usePlayer } from "@/lib/player-context";
import type { Track } from "@/lib/types";

export function TrackMenuButton({ track }: { track: Track }) {
  const { openContextMenu } = usePlayer();
  return (
    <Pressable
      hitSlop={10}
      onPress={(e) => {
        e.stopPropagation();
        openContextMenu(track);
      }}
      className="px-1 py-1"
    >
      <Ionicons name="ellipsis-vertical" size={18} color={Colors.textMuted} />
    </Pressable>
  );
}
