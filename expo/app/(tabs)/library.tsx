import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { useMemo, useState } from "react";
import {
  FlatList,
  Modal,
  Pressable,
  ScrollView,
  Text,
  TextInput,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { TrackMenuButton } from "@/components/track-menu-button";
import { Colors } from "@/constants/theme";
import {
  ALBUMS,
  ALL_SONGS,
  ARTISTS,
  PLAYLISTS,
  formatDuration,
} from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";
import type { LibrarySection } from "@/lib/types";

const SECTIONS: { id: LibrarySection; label: string }[] = [
  { id: "playlists", label: "Playlists" },
  { id: "songs", label: "Songs" },
  { id: "albums", label: "Albums" },
  { id: "artists", label: "Artists" },
  { id: "liked", label: "Liked" },
];

export default function LibraryScreen() {
  const [section, setSection] = useState<LibrarySection>("playlists");
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const { liked, jumpTo, queue, userPlaylists } = usePlayer();

  const q = searchQuery.trim().toLowerCase();
  const allPlaylists = useMemo(
    () => [...userPlaylists, ...PLAYLISTS],
    [userPlaylists],
  );
  const filteredPlaylists = useMemo(
    () =>
      q
        ? allPlaylists.filter((p) => p.name.toLowerCase().includes(q))
        : allPlaylists,
    [q, allPlaylists],
  );
  const filteredSongs = useMemo(
    () =>
      q
        ? ALL_SONGS.filter(
            (t) =>
              t.title.toLowerCase().includes(q) ||
              t.artist.toLowerCase().includes(q),
          )
        : ALL_SONGS,
    [q],
  );
  const filteredAlbums = useMemo(
    () =>
      q
        ? ALBUMS.filter(
            (a) =>
              a.title.toLowerCase().includes(q) ||
              a.artist.toLowerCase().includes(q),
          )
        : ALBUMS,
    [q],
  );
  const filteredArtists = useMemo(
    () => (q ? ARTISTS.filter((a) => a.name.toLowerCase().includes(q)) : ARTISTS),
    [q],
  );

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-text-primary text-[26px] font-extrabold font-sans">
          Your Library
        </Text>
        <View className="flex-row gap-3">
          <Pressable
            hitSlop={10}
            onPress={() => {
              setSearchOpen((v) => {
                if (v) setSearchQuery("");
                return !v;
              });
            }}
          >
            <Ionicons
              name={searchOpen ? "close" : "search-outline"}
              size={22}
              color={Colors.textPrimary}
            />
          </Pressable>
          <Pressable hitSlop={10} onPress={() => setCreateOpen(true)}>
            <Ionicons name="add" size={26} color={Colors.textPrimary} />
          </Pressable>
        </View>
      </View>

      {searchOpen ? (
        <View className="px-4 pb-3">
          <View className="flex-row items-center bg-bg-card rounded-md px-3 h-10 gap-2">
            <Ionicons name="search" size={16} color={Colors.textMuted} />
            <TextInput
              autoFocus
              value={searchQuery}
              onChangeText={setSearchQuery}
              placeholder="Search in library"
              placeholderTextColor={Colors.textMuted}
              className="flex-1 text-text-primary text-sm font-sans"
            />
            {searchQuery.length > 0 ? (
              <Pressable hitSlop={6} onPress={() => setSearchQuery("")}>
                <Ionicons
                  name="close-circle"
                  size={16}
                  color={Colors.textMuted}
                />
              </Pressable>
            ) : null}
          </View>
        </View>
      ) : null}

      <View className="h-12 mb-2">
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={{
            paddingHorizontal: 16,
            alignItems: "center",
            gap: 8,
          }}
        >
          {SECTIONS.map((s) => {
            const active = s.id === section;
            return (
              <Pressable
                key={s.id}
                onPress={() => setSection(s.id)}
                className={`h-8 px-3.5 rounded-full items-center justify-center ${active ? "bg-accent" : "bg-bg-card"}`}
              >
                <Text
                  className={`text-text-primary text-[13px] font-sans ${active ? "font-bold" : "font-medium"}`}
                >
                  {s.label}
                </Text>
              </Pressable>
            );
          })}
        </ScrollView>
      </View>

      {section === "playlists" ? (
        <FlatList
          key="list-playlists"
          data={filteredPlaylists}
          keyExtractor={(p) => p.id}
          contentContainerStyle={{
            paddingHorizontal: 16,
            paddingBottom: 24,
            gap: 8,
          }}
          renderItem={({ item }) => (
            <Pressable
              onPress={() => router.push(`/playlist/${item.id}`)}
              className="flex-row items-center gap-3 py-1.5 active:opacity-80"
            >
              <Image
                source={item.artwork}
                className="w-[60px] h-[60px] rounded-md"
                contentFit="cover"
              />
              <View className="flex-1">
                <Text
                  numberOfLines={1}
                  className="text-text-primary text-[15px] font-semibold font-sans"
                >
                  {item.name}
                </Text>
                <Text
                  numberOfLines={1}
                  className="text-text-secondary text-[13px] mt-0.5 font-sans"
                >
                  {item.isSmart ? "Smart playlist" : "Playlist"} •{" "}
                  {item.trackCount} tracks
                </Text>
              </View>
            </Pressable>
          )}
        />
      ) : section === "songs" ? (
        <FlatList
          key="list-songs"
          data={filteredSongs}
          keyExtractor={(t) => t.id}
          contentContainerStyle={{ paddingBottom: 24 }}
          renderItem={({ item, index }) => (
            <Pressable
              onPress={() => jumpTo(index)}
              className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover"
            >
              {item.artwork ? (
                <Image
                  source={item.artwork}
                  className="w-12 h-12 rounded"
                  contentFit="cover"
                />
              ) : (
                <View className="w-12 h-12 bg-bg-card rounded items-center justify-center">
                  <Ionicons
                    name="musical-note"
                    size={18}
                    color={Colors.textMuted}
                  />
                </View>
              )}
              <View className="flex-1">
                <Text
                  numberOfLines={1}
                  className="text-text-primary text-sm font-medium font-sans"
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
            </Pressable>
          )}
        />
      ) : section === "albums" ? (
        <FlatList
          key="list-albums"
          data={filteredAlbums}
          keyExtractor={(a) => a.id}
          numColumns={2}
          columnWrapperStyle={{ gap: 12, paddingHorizontal: 16 }}
          contentContainerStyle={{ paddingBottom: 24, gap: 16 }}
          renderItem={({ item }) => (
            <Pressable
              onPress={() => router.push(`/album/${item.id}`)}
              className="flex-1 max-w-[48%] active:opacity-80"
            >
              <Image
                source={item.artwork}
                className="w-full aspect-square rounded-md"
                contentFit="cover"
              />
              <Text
                numberOfLines={1}
                className="text-text-primary text-[13px] font-semibold mt-1.5 font-sans"
              >
                {item.title}
              </Text>
              <Text
                numberOfLines={1}
                className="text-text-secondary text-xs font-sans"
              >
                {item.artist}
              </Text>
            </Pressable>
          )}
        />
      ) : section === "artists" ? (
        <FlatList
          key="list-artists"
          data={filteredArtists}
          keyExtractor={(a) => a.id}
          contentContainerStyle={{
            paddingHorizontal: 16,
            paddingBottom: 24,
            gap: 8,
          }}
          renderItem={({ item }) => (
            <Pressable
              onPress={() => router.push(`/artist/${item.id}`)}
              className="flex-row items-center gap-3.5 py-1.5 active:opacity-80"
            >
              <Image
                source={item.image}
                className="w-14 h-14 rounded-full"
                contentFit="cover"
              />
              <View className="flex-1">
                <Text className="text-text-primary text-[15px] font-semibold font-sans">
                  {item.name}
                </Text>
                <Text className="text-text-secondary text-[13px] mt-0.5 font-sans">
                  Artist
                </Text>
              </View>
            </Pressable>
          )}
        />
      ) : (
        <FlatList
          key="list-liked"
          data={filteredSongs.filter((t) => liked.has(t.id))}
          keyExtractor={(t) => t.id}
          ListHeaderComponent={
            <View className="px-4 pt-1 pb-4">
              <View className="flex-row items-center gap-3.5">
                <View className="w-20 h-20 rounded-md bg-accent items-center justify-center">
                  <Ionicons name="heart" size={36} color="#FFFFFF" />
                </View>
                <View className="flex-1">
                  <Text className="text-text-primary text-[22px] font-extrabold font-sans">
                    Liked Songs
                  </Text>
                  <Text className="text-text-secondary text-[13px] mt-1 font-sans">
                    {liked.size} liked tracks
                  </Text>
                </View>
              </View>
            </View>
          }
          ListEmptyComponent={
            <Text className="text-text-secondary px-4 font-sans">
              No liked songs yet. Tap the heart on any track.
            </Text>
          }
          renderItem={({ item }) => {
            const idx = queue.findIndex((t) => t.id === item.id);
            return (
              <Pressable
                onPress={() => idx >= 0 && jumpTo(idx)}
                className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover"
              >
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
                    className="text-text-primary text-sm font-medium font-sans"
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
                <TrackMenuButton track={item} />
              </Pressable>
            );
          }}
        />
      )}

      <Modal
        visible={createOpen}
        transparent
        animationType="fade"
        onRequestClose={() => setCreateOpen(false)}
      >
        <Pressable
          className="flex-1 bg-black/60"
          onPress={() => setCreateOpen(false)}
        >
          <Pressable
            onPress={(e) => e.stopPropagation()}
            className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7"
          >
            <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />
            <Text className="text-text-primary text-base font-bold text-center py-2 font-sans">
              Create new
            </Text>
            {[
              {
                icon: "musical-notes-outline",
                label: "Playlist",
                desc: "Build a custom mix",
                href: "/playlist/new",
              },
              {
                icon: "list-outline",
                label: "Smart playlist",
                desc: "Auto-updates from rules",
                href: "/playlist/new?mode=smart",
              },
            ].map((item) => (
              <Pressable
                key={item.label}
                onPress={() => {
                  setCreateOpen(false);
                  setTimeout(() => router.push(item.href as any), 50);
                }}
                android_ripple={{ color: Colors.bgHover }}
                className="flex-row items-center px-5 py-3.5 gap-4 active:bg-bg-hover"
              >
                <View className="w-11 h-11 rounded-full bg-bg-card items-center justify-center">
                  <Ionicons
                    name={
                      item.icon as React.ComponentProps<typeof Ionicons>["name"]
                    }
                    size={22}
                    color={Colors.textPrimary}
                  />
                </View>
                <View className="flex-1">
                  <Text className="text-text-primary text-[15px] font-semibold font-sans">
                    {item.label}
                  </Text>
                  <Text className="text-text-secondary text-xs mt-0.5 font-sans">
                    {item.desc}
                  </Text>
                </View>
              </Pressable>
            ))}
          </Pressable>
        </Pressable>
      </Modal>
    </SafeAreaView>
  );
}
