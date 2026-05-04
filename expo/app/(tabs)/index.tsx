import { Ionicons } from "@expo/vector-icons";
import { router } from "expo-router";
import { Pressable, ScrollView, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { CardRow } from "@/components/card-row";
import { NotConnectedState } from "@/components/empty-state";
import { PlaylistCover } from "@/components/playlist-cover";
import { SectionHeader } from "@/components/section-header";
import { Colors } from "@/constants/theme";
import { useIsConnected } from "@/lib/connection";
import {
  useLibraryAlbums,
  useLibraryArtists,
  useLibraryPlaylists,
} from "@/lib/library-source";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

export default function HomeScreen() {
  // All sections back-by real gRPC reads. The "Recently played" / "Made
  // for you" / "Top artists" / "Popular albums" buckets are slices of the
  // real catalog for now — backend filters land later.
  const isConnected = useIsConnected();
  const { data: albums } = useLibraryAlbums();
  const { data: artists } = useLibraryArtists();
  const { data: playlists } = useLibraryPlaylists();
  const bottomPad = useBottomSpacing(24);

  const quickPicks = playlists.slice(0, 6);
  const recentlyPlayed = albums.slice(0, 8);
  const madeForYou = playlists.filter((p) => p.isSmart).slice(0, 12);
  const topArtists = artists.slice(0, 12);
  const popularAlbums = albums.slice(0, 12);

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <ScrollView
        contentContainerStyle={{ paddingBottom: bottomPad }}
        showsVerticalScrollIndicator={false}
      >
        <View className="px-4 pt-2 pb-4 flex-row items-center justify-between">
          <Text className="text-text-primary text-[26px] font-display-extra">
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
        {quickPicks.length > 0 ? (
          <View className="px-4 flex-row flex-wrap gap-2 mb-6">
            {quickPicks.map((p) => (
              <Pressable
                key={p.id}
                onPress={() => router.push(`/playlist/${p.id}`)}
                className="w-[48.5%] flex-row items-center bg-bg-card rounded-md overflow-hidden active:bg-bg-hover"
              >
                <PlaylistCover
                  artwork={p.artwork}
                  seed={p.id || p.name}
                  size={56}
                  rounded="sm"
                  iconSize={22}
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
        ) : null}

        {recentlyPlayed.length > 0 ? (
          <>
            <SectionHeader title="Recently played" />
            <View className="mb-7">
              <CardRow
                data={recentlyPlayed.map((a) => ({
                  id: a.id,
                  title: a.title,
                  subtitle: a.artist,
                  image: a.artwork,
                  placeholderIcon: "disc",
                }))}
                onPress={(item) => router.push(`/album/${item.id}`)}
              />
            </View>
          </>
        ) : null}

        {madeForYou.length > 0 ? (
          <>
            <SectionHeader
              title="Made for you"
              subtitle="Smart playlists refreshed automatically"
            />
            <View className="mb-7">
              <CardRow
                data={madeForYou.map((p) => ({
                  id: p.id,
                  title: p.name,
                  subtitle: p.description,
                  image: p.artwork,
                  placeholderIcon: "flash",
                  colorfulPlaceholder: true,
                }))}
                onPress={(item) => router.push(`/playlist/${item.id}`)}
              />
            </View>
          </>
        ) : null}

        {topArtists.length > 0 ? (
          <>
            <SectionHeader title="Your top artists" />
            <View className="mb-7">
              <CardRow
                size={130}
                data={topArtists.map((a) => ({
                  id: a.id,
                  title: a.name,
                  subtitle: "Artist",
                  image: a.image,
                  rounded: "full" as const,
                  placeholderIcon: "person",
                }))}
                onPress={(item) => router.push(`/artist/${item.id}`)}
              />
            </View>
          </>
        ) : null}

        {popularAlbums.length > 0 ? (
          <>
            <SectionHeader title="Popular albums" />
            <CardRow
              data={popularAlbums.map((a) => ({
                id: a.id,
                title: a.title,
                subtitle: `${a.artist}${a.year ? ` • ${a.year}` : ""}`,
                image: a.artwork,
                placeholderIcon: "disc",
              }))}
              onPress={(item) => router.push(`/album/${item.id}`)}
            />
          </>
        ) : null}

        {!isConnected ? (
          <NotConnectedState />
        ) : albums.length === 0 &&
          artists.length === 0 &&
          playlists.length === 0 ? (
          <View className="px-6 pt-12 items-center">
            <Ionicons
              name="musical-notes-outline"
              size={48}
              color={Colors.textMuted}
            />
            <Text className="text-text-secondary text-[14px] text-center mt-4 font-sans">
              Library is empty — wait for the daemon to finish scanning.
            </Text>
          </View>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  );
}
