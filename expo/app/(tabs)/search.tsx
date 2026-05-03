import { Ionicons } from "@expo/vector-icons";
import { router } from "expo-router";
import { useMemo, useState } from "react";
import { Pressable, ScrollView, Text, TextInput, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { TrackMenuButton } from "@/components/track-menu-button";
import { Colors } from "@/constants/theme";
import { ALL_SONGS, GENRES, formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

export default function SearchScreen() {
  const [query, setQuery] = useState("");
  const { jumpTo, queue } = usePlayer();

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return ALL_SONGS.filter(
      (t) =>
        t.title.toLowerCase().includes(q) ||
        t.artist.toLowerCase().includes(q) ||
        t.album.toLowerCase().includes(q),
    );
  }, [query]);

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <View className="px-4 pt-2 pb-4">
        <Text className="text-text-primary text-[26px] font-extrabold mb-4 font-sans">
          Search
        </Text>
        <View className="flex-row items-center bg-bg-card rounded-md px-3 h-11 gap-2">
          <Ionicons name="search" size={18} color={Colors.textMuted} />
          <TextInput
            value={query}
            onChangeText={setQuery}
            placeholder="Songs, artists, albums"
            placeholderTextColor={Colors.textMuted}
            className="flex-1 text-text-primary text-[15px] font-sans"
            autoCorrect={false}
            returnKeyType="search"
          />
          {query.length > 0 ? (
            <Pressable hitSlop={6} onPress={() => setQuery("")}>
              <Ionicons
                name="close-circle"
                size={18}
                color={Colors.textMuted}
              />
            </Pressable>
          ) : null}
        </View>
      </View>

      {query.trim().length > 0 ? (
        <ScrollView contentContainerStyle={{ paddingBottom: 24 }}>
          {results.length === 0 ? (
            <Text className="text-text-secondary px-4 mt-6 font-sans">
              No results for &ldquo;{query}&rdquo;
            </Text>
          ) : (
            results.map((track) => {
              const idx = queue.findIndex((t) => t.id === track.id);
              return (
                <Pressable
                  key={track.id}
                  onPress={() => idx >= 0 && jumpTo(idx)}
                  className="flex-row items-center px-4 py-2.5 gap-3 active:bg-bg-hover"
                >
                  <View className="w-11 h-11 bg-bg-card rounded-md items-center justify-center">
                    <Ionicons
                      name="musical-note"
                      size={18}
                      color={Colors.textMuted}
                    />
                  </View>
                  <View className="flex-1">
                    <Text
                      numberOfLines={1}
                      className="text-text-primary text-sm font-semibold font-sans"
                    >
                      {track.title}
                    </Text>
                    <Text
                      numberOfLines={1}
                      className="text-text-secondary text-xs font-sans"
                    >
                      {track.artist} • {track.album}
                    </Text>
                  </View>
                  <Text className="text-text-muted text-xs font-mono">
                    {formatDuration(track.duration)}
                  </Text>
                  <TrackMenuButton track={track} />
                </Pressable>
              );
            })
          )}
        </ScrollView>
      ) : (
        <ScrollView
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: 24 }}
          showsVerticalScrollIndicator={false}
        >
          <Text className="text-text-primary text-lg font-bold mb-3 font-sans">
            Browse all
          </Text>
          <View className="flex-row flex-wrap gap-2">
            {GENRES.map((g) => (
              <Pressable
                key={g.id}
                onPress={() => router.push(`/genre/${g.id}`)}
                style={{ backgroundColor: g.color }}
                className="w-[48.5%] h-[100px] rounded-md p-3 overflow-hidden active:opacity-80"
              >
                <Text className="text-white text-lg font-bold font-sans">
                  {g.name}
                </Text>
              </Pressable>
            ))}
          </View>
        </ScrollView>
      )}
    </SafeAreaView>
  );
}
