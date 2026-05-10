import { Ionicons } from "@expo/vector-icons";
import { IconCast, IconDevices } from "@tabler/icons-react-native";
import { router } from "expo-router";
import { useMemo, useState } from "react";
import { Pressable, ScrollView, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { CardRow } from "@/components/card-row";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { DevicePickerSheet } from "@/components/device-picker";
import { NotConnectedState } from "@/components/empty-state";
import { PlaylistCover } from "@/components/playlist-cover";
import { SectionHeader } from "@/components/section-header";
import { Colors } from "@/constants/theme";
import { useIsConnected } from "@/lib/connection";
import {
  useLibraryArtists,
  useLibraryPlaylists,
  useLibraryTracks,
} from "@/lib/library-source";
import type { Album, Track } from "@/lib/types";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

export default function HomeScreen() {
  // No backend "recently played" / "top albums" RPC yet — derive both from
  // the track list, mirroring the GPUI client:
  //   Recently played → albums sorted by max track id desc (cuids are
  //                     time-ordered, so MAX(id) ≈ date_added DESC).
  //   Popular albums  → albums sorted by track count desc, name asc tiebreak.
  // Replace with real RPCs once the backend exposes play counts / last_played.
  const isConnected = useIsConnected();
  const { data: tracks } = useLibraryTracks();
  const { data: artists } = useLibraryArtists();
  const { data: playlists } = useLibraryPlaylists();
  const bottomPad = useBottomSpacing(24);

  const { recentlyPlayed, popularAlbums } = useMemo(
    () => aggregateAlbums(tracks),
    [tracks],
  );
  const [castOpen, setCastOpen] = useState(false);

  const quickPicks = playlists.slice(0, 6);
  const madeForYou = playlists.filter((p) => p.isSmart).slice(0, 12);
  const topArtists = artists.slice(0, 12);
  const hasAnyAlbum =
    recentlyPlayed.length > 0 || popularAlbums.length > 0;

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <RemoteServerBanner />
      <ScrollView
        contentContainerStyle={{ paddingBottom: bottomPad }}
        showsVerticalScrollIndicator={false}
      >
        <View className="px-4 pt-2 pb-4 flex-row items-center justify-between">
          <Text className="text-text-primary text-[26px] font-display-extra">
            Home
          </Text>
          <View className="flex-row items-center gap-2">
            <Pressable
              hitSlop={8}
              onPress={() => router.push("/settings/server")}
              accessibilityLabel="Rockbox server"
              className="w-9 h-9 rounded-full bg-bg-card items-center justify-center"
            >
              <IconDevices
                size={20}
                color={Colors.textPrimary}
                strokeWidth={1.75}
              />
            </Pressable>
            <Pressable
              hitSlop={8}
              onPress={() => setCastOpen(true)}
              accessibilityLabel="AirPlay & Cast"
              className="w-9 h-9 rounded-full bg-bg-card items-center justify-center"
            >
              <IconCast
                size={20}
                color={Colors.textPrimary}
                strokeWidth={1.75}
              />
            </Pressable>
            <Pressable
              hitSlop={8}
              onPress={() => router.push("/settings")}
              accessibilityLabel="Settings"
              className="w-9 h-9 rounded-full bg-bg-card items-center justify-center"
            >
              <Ionicons
                name="settings-outline"
                size={20}
                color={Colors.textPrimary}
              />
            </Pressable>
          </View>
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
                  numberOfLines={1}
                  ellipsizeMode="tail"
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
        ) : !hasAnyAlbum && artists.length === 0 && playlists.length === 0 ? (
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

      <DevicePickerSheet
        visible={castOpen}
        onClose={() => setCastOpen(false)}
      />
    </SafeAreaView>
  );
}

type Agg = {
  albumName: string;
  artist: string;
  artwork: string;
  trackCount: number;
  maxId: string;
};

function aggregateAlbums(tracks: Track[]): {
  recentlyPlayed: Album[];
  popularAlbums: Album[];
} {
  const map = new Map<string, Agg>();
  for (const t of tracks) {
    if (!t.album) continue;
    const key = t.albumId || t.album;
    const entry =
      map.get(key) ??
      ({
        albumName: t.album,
        artist: t.artist,
        artwork: "",
        trackCount: 0,
        maxId: "",
      } as Agg);
    entry.trackCount += 1;
    if (!entry.artwork && t.artwork) entry.artwork = t.artwork;
    if (t.id > entry.maxId) entry.maxId = t.id;
    map.set(key, entry);
  }

  const make = (id: string, a: Agg): Album => ({
    id,
    title: a.albumName,
    artist: a.artist,
    artwork: a.artwork,
  });

  const entries = Array.from(map.entries());
  const recentlyPlayed = [...entries]
    .sort((x, y) => (y[1].maxId > x[1].maxId ? 1 : -1))
    .slice(0, 8)
    .map(([id, a]) => make(id, a));
  const popularAlbums = [...entries]
    .sort((x, y) =>
      x[1].trackCount !== y[1].trackCount
        ? y[1].trackCount - x[1].trackCount
        : x[1].albumName.localeCompare(y[1].albumName),
    )
    .slice(0, 12)
    .map(([id, a]) => make(id, a));

  return { recentlyPlayed, popularAlbums };
}
