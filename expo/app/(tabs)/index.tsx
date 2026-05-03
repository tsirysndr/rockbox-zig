import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { Pressable, ScrollView, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { CardRow } from "@/components/card-row";
import { SectionHeader } from "@/components/section-header";
import { Colors } from "@/constants/theme";
import {
  ALBUMS,
  MADE_FOR_YOU,
  PLAYLISTS,
  RECENTLY_PLAYED,
  TOP_ARTISTS,
} from "@/lib/mock-data";

export default function HomeScreen() {
  const quickPicks = PLAYLISTS.slice(0, 6);

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <ScrollView
        contentContainerStyle={{ paddingBottom: 24 }}
        showsVerticalScrollIndicator={false}
      >
        <View className="px-4 pt-2 pb-4 flex-row items-center justify-between">
          <Text className="text-text-primary text-[26px] font-extrabold font-sans">
            Home
          </Text>
          <Pressable
            hitSlop={8}
            onPress={() => router.push("/settings")}
            className="w-9 h-9 rounded-full bg-bg-card items-center justify-center"
          >
            <Ionicons
              name="settings-outline"
              size={20}
              color={Colors.textPrimary}
            />
          </Pressable>
        </View>

        {/* Quick picks 2-column grid (Spotify-style) */}
        <View className="px-4 flex-row flex-wrap gap-2 mb-6">
          {quickPicks.map((p) => (
            <Pressable
              key={p.id}
              onPress={() => router.push(`/playlist/${p.id}`)}
              className="w-[48.5%] flex-row items-center bg-bg-card rounded-md overflow-hidden active:bg-bg-hover"
            >
              <Image
                source={p.artwork}
                className="w-14 h-14"
                contentFit="cover"
              />
              <Text
                numberOfLines={2}
                className="flex-1 text-text-primary text-[13px] font-semibold px-2.5 font-sans"
              >
                {p.name}
              </Text>
            </Pressable>
          ))}
        </View>

        <SectionHeader title="Recently played" />
        <View className="mb-7">
          <CardRow
            data={RECENTLY_PLAYED.map((a) => ({
              id: a.id,
              title: a.title,
              subtitle: a.artist,
              image: a.artwork,
            }))}
            onPress={(item) => router.push(`/album/${item.id}`)}
          />
        </View>

        <SectionHeader
          title="Made for you"
          subtitle="Curated mixes refreshed daily"
        />
        <View className="mb-7">
          <CardRow
            data={MADE_FOR_YOU.map((p) => ({
              id: p.id,
              title: p.name,
              subtitle: p.description,
              image: p.artwork,
            }))}
            onPress={(item) => router.push(`/playlist/${item.id}`)}
          />
        </View>

        <SectionHeader title="Your top artists" />
        <View className="mb-7">
          <CardRow
            size={130}
            data={TOP_ARTISTS.map((a) => ({
              id: a.id,
              title: a.name,
              subtitle: "Artist",
              image: a.image,
              rounded: "full" as const,
            }))}
            onPress={(item) => router.push(`/artist/${item.id}`)}
          />
        </View>

        <SectionHeader title="Popular albums" />
        <CardRow
          data={ALBUMS.map((a) => ({
            id: a.id,
            title: a.title,
            subtitle: `${a.artist}${a.year ? ` • ${a.year}` : ""}`,
            image: a.artwork,
          }))}
          onPress={(item) => router.push(`/album/${item.id}`)}
        />
      </ScrollView>
    </SafeAreaView>
  );
}
