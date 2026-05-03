import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { Pressable, Text, View } from "react-native";
import { Colors } from "@/constants/theme";
import { usePlayer } from "@/lib/player-context";

export function MiniPlayer() {
  const { currentTrack, isPlaying, toggle, next } = usePlayer();

  if (!currentTrack) return null;

  return (
    <View>
      <Pressable
        onPress={() => router.push("/player")}
        className="flex-row items-center px-4 py-3.5 gap-3.5"
      >
        {currentTrack.artwork ? (
          <Image
            source={currentTrack.artwork}
            className="w-11 h-11 rounded-md"
            contentFit="cover"
          />
        ) : (
          <View className="w-11 h-11 rounded-md bg-bg-card items-center justify-center">
            <Ionicons name="musical-note" size={18} color={Colors.textMuted} />
          </View>
        )}
        <View className="flex-1 min-w-0">
          <Text
            numberOfLines={1}
            className="text-text-primary text-[13px] font-semibold font-sans"
          >
            {currentTrack.title}
          </Text>
          <Text
            numberOfLines={1}
            className="text-text-secondary text-[11px] font-sans"
          >
            {currentTrack.artist}
          </Text>
        </View>
        <Pressable
          hitSlop={10}
          onPress={(e) => {
            e.stopPropagation();
            toggle();
          }}
          className="px-1.5"
        >
          <Ionicons
            name={isPlaying ? "pause" : "play"}
            size={24}
            color={Colors.textPrimary}
          />
        </Pressable>
        <Pressable
          hitSlop={10}
          onPress={(e) => {
            e.stopPropagation();
            next();
          }}
          className="px-1.5"
        >
          <Ionicons
            name="play-skip-forward"
            size={20}
            color={Colors.textPrimary}
          />
        </Pressable>
      </Pressable>
    </View>
  );
}
