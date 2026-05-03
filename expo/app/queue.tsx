import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { FlatList, Pressable, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { TrackMenuButton } from "@/components/track-menu-button";
import { Colors } from "@/constants/theme";
import { formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

export default function QueueScreen() {
  const { queue, currentIdx, jumpTo, removeFromQueue, isPlaying } = usePlayer();

  return (
    <SafeAreaView className="flex-1 bg-bg">
      <View className="flex-row items-center justify-between px-4 py-3 border-b border-divider">
        <Pressable hitSlop={8} onPress={() => router.back()}>
          <Ionicons name="chevron-down" size={26} color={Colors.textPrimary} />
        </Pressable>
        <Text className="text-text-primary text-base font-bold font-sans">
          Queue
        </Text>
        <Text className="text-text-secondary text-xs font-sans">
          {currentIdx + 1} / {queue.length}
        </Text>
      </View>

      <FlatList
        data={queue}
        keyExtractor={(t, i) => `${t.id}-${i}`}
        renderItem={({ item, index }) => {
          const isCurrent = index === currentIdx;
          return (
            <Pressable
              onPress={() => jumpTo(index)}
              className={`flex-row items-center gap-3 px-4 py-2.5 ${isCurrent ? "bg-accent-soft border-l-[3px] border-accent" : "active:bg-bg-hover"}`}
            >
              <View className="w-7 items-center">
                {isCurrent ? (
                  <Ionicons
                    name={isPlaying ? "musical-notes" : "pause"}
                    size={16}
                    color={Colors.accent}
                  />
                ) : (
                  <Text className="text-text-muted text-xs font-mono">
                    {index + 1}
                  </Text>
                )}
              </View>
              {item.artwork ? (
                <Image
                  source={item.artwork}
                  className="w-11 h-11 rounded"
                  contentFit="cover"
                />
              ) : null}
              <View className="flex-1">
                <Text
                  numberOfLines={1}
                  className={`text-text-primary text-sm font-sans ${isCurrent ? "font-bold" : "font-medium"}`}
                >
                  {item.title}
                </Text>
                <Text
                  numberOfLines={1}
                  className="text-text-secondary text-xs font-sans"
                >
                  {item.artist}
                </Text>
              </View>
              <Text className="text-text-muted text-xs font-mono">
                {formatDuration(item.duration)}
              </Text>
              <TrackMenuButton track={item} />
              <Pressable
                hitSlop={8}
                onPress={() => removeFromQueue(index)}
                className="p-1"
              >
                <Ionicons name="close" size={18} color={Colors.textMuted} />
              </Pressable>
            </Pressable>
          );
        }}
      />
    </SafeAreaView>
  );
}
