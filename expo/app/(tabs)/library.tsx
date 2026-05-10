import { Ionicons } from "@expo/vector-icons";
import { useQueryClient } from "@tanstack/react-query";
import { Image } from "expo-image";
import { router } from "expo-router";
import { useCallback, useMemo, useState } from "react";
import {
  FlatList,
  Modal,
  Pressable,
  RefreshControl,
  ScrollView,
  Text,
  TextInput,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { NotConnectedState } from "@/components/empty-state";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { EqualizerBars } from "@/components/equalizer-bars";
import { PlaylistCover } from "@/components/playlist-cover";
import { TrackMenuButton } from "@/components/track-menu-button";
import { useIsConnected } from "@/lib/connection";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import { formatDuration } from "@/lib/mock-data";
import { qk } from "@/lib/queries";
import { RockboxClient } from "@/lib/rockbox-client";
import {
  useLibraryAlbums,
  useLibraryArtists,
  useLibraryLikedTracks,
  useLibraryPlaylists,
  useLibraryTracks,
} from "@/lib/library-source";
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
  const [refreshing, setRefreshing] = useState(false);
  const { liked, userPlaylists, playTrack, playQueue, currentTrack, isPlaying } =
    usePlayer();
  const isConnected = useIsConnected();
  const qc = useQueryClient();

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    RockboxClient.rescanLibrary();
    await qc.invalidateQueries({
      predicate: (q) => {
        const k = q.queryKey;
        if (!Array.isArray(k) || k.length < 2) return false;
        const type = k[1];
        // Leave playback and discovery caches alone — only refresh library data.
        return (
          type !== "status" &&
          type !== "currentTrack" &&
          type !== "playlist" &&
          type !== "discoveredServers" &&
          type !== "outputDevices"
        );
      },
    });
    setRefreshing(false);
  }, [qc]);
  const bottomPad = useBottomSpacing(24);
  const { data: tracks } = useLibraryTracks();
  const { data: albums } = useLibraryAlbums();
  const { data: artists } = useLibraryArtists();
  const { data: playlists } = useLibraryPlaylists();
  const { data: likedTracks } = useLibraryLikedTracks();

  const q = searchQuery.trim().toLowerCase();
  const allPlaylists = useMemo(
    () => [...userPlaylists, ...playlists],
    [userPlaylists, playlists],
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
        ? tracks.filter(
            (t) =>
              t.title.toLowerCase().includes(q) ||
              t.artist.toLowerCase().includes(q),
          )
        : tracks,
    [q, tracks],
  );
  const filteredAlbums = useMemo(
    () =>
      q
        ? albums.filter(
            (a) =>
              a.title.toLowerCase().includes(q) ||
              a.artist.toLowerCase().includes(q),
          )
        : albums,
    [q, albums],
  );
  const filteredArtists = useMemo(
    () => (q ? artists.filter((a) => a.name.toLowerCase().includes(q)) : artists),
    [q, artists],
  );
  // likedTracks comes back from the server already ordered by liked-at desc
  // (`favourites.created_at DESC` in repo::favourites::all_tracks). Filter
  // for the search box but never re-sort.
  const filteredLiked = useMemo(
    () =>
      q
        ? likedTracks.filter(
            (t) =>
              t.title.toLowerCase().includes(q) ||
              t.artist.toLowerCase().includes(q),
          )
        : likedTracks,
    [q, likedTracks],
  );

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <RemoteServerBanner />
      <View className="px-4 pt-2 pb-3 flex-row items-center justify-between">
        <Text className="text-text-primary text-[26px] font-display-extra">
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

      {!isConnected ? (
        <NotConnectedState />
      ) : section === "playlists" ? (
        <FlatList
          key="list-playlists"
          data={filteredPlaylists}
          keyExtractor={(p) => p.id}
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={onRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
            />
          }
          contentContainerStyle={{
            paddingHorizontal: 16,
            paddingBottom: bottomPad,
            gap: 8,
          }}
          renderItem={({ item }) => (
            <Pressable
              onPress={() => router.push(`/playlist/${item.id}`)}
              className="flex-row items-center gap-3 py-1.5 active:opacity-80"
            >
              <PlaylistCover
                artwork={item.artwork}
                seed={item.id || item.name}
                size={60}
                rounded="md"
                iconSize={24}
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
                  {item.isSmart ? "Smart playlist" : "Playlist"}
                  {item.trackCount > 0 ? ` • ${item.trackCount} tracks` : ""}
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
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={onRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
            />
          }
          contentContainerStyle={{ paddingBottom: bottomPad }}
          renderItem={({ item }) => {
            const isCurrent = currentTrack?.id === item.id && !!item.id;
            return (
              <Pressable
                onPress={() => playTrack(item)}
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
                    className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
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
                {isCurrent ? (
                  <EqualizerBars size={14} playing={isPlaying} />
                ) : null}
                <TrackMenuButton track={item} />
              </Pressable>
            );
          }}
        />
      ) : section === "albums" ? (
        <FlatList
          key="list-albums"
          data={filteredAlbums}
          keyExtractor={(a) => a.id}
          numColumns={2}
          columnWrapperStyle={{ gap: 12, paddingHorizontal: 16 }}
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={onRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
            />
          }
          contentContainerStyle={{ paddingBottom: bottomPad, gap: 16 }}
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
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={onRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
            />
          }
          contentContainerStyle={{
            paddingHorizontal: 16,
            paddingBottom: bottomPad,
            gap: 8,
          }}
          renderItem={({ item }) => (
            <Pressable
              onPress={() => router.push(`/artist/${item.id}`)}
              className="flex-row items-center gap-3.5 py-1.5 active:opacity-80"
            >
              {item.image ? (
                <Image
                  source={item.image}
                  className="w-14 h-14 rounded-full"
                  contentFit="cover"
                />
              ) : (
                <View className="w-14 h-14 rounded-full bg-bg-card items-center justify-center">
                  <Ionicons name="person" size={20} color={Colors.textMuted} />
                </View>
              )}
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
          data={filteredLiked}
          keyExtractor={(t) => t.id}
          refreshControl={
            <RefreshControl
              refreshing={refreshing}
              onRefresh={onRefresh}
              tintColor={Colors.accent}
              colors={[Colors.accent]}
            />
          }
          ListHeaderComponent={
            <View className="px-4 pt-1 pb-4">
              <View className="flex-row items-center gap-3.5">
                <View className="w-20 h-20 rounded-md bg-accent items-center justify-center">
                  <Ionicons name="heart" size={36} color="#FFFFFF" />
                </View>
                <View className="flex-1">
                  <Text className="text-text-primary text-[22px] font-display-extra">
                    Liked Songs
                  </Text>
                  <Text className="text-text-secondary text-[13px] mt-1 font-sans">
                    {likedTracks.length} liked tracks
                  </Text>
                </View>
              </View>
              <View className="flex-row items-center gap-4 mt-4">
                <Pressable
                  hitSlop={6}
                  onPress={() => playQueue(likedTracks, { shuffle: true })}
                  disabled={likedTracks.length === 0}
                  className="active:opacity-70 disabled:opacity-40"
                >
                  <Ionicons
                    name="shuffle"
                    size={26}
                    color={
                      likedTracks.length === 0
                        ? Colors.textMuted
                        : Colors.textPrimary
                    }
                  />
                </Pressable>
                <View className="flex-1" />
                <Pressable
                  onPress={() => playQueue(likedTracks)}
                  disabled={likedTracks.length === 0}
                  className="w-14 h-14 rounded-full items-center justify-center bg-accent active:opacity-85 disabled:opacity-40"
                  style={{
                    shadowColor: Colors.accent,
                    shadowOpacity: 0.5,
                    shadowRadius: 14,
                    shadowOffset: { width: 0, height: 6 },
                  }}
                >
                  <Ionicons
                    name="play"
                    size={26}
                    color="#FFFFFF"
                    style={{ marginLeft: 3 }}
                  />
                </Pressable>
              </View>
            </View>
          }
          ListEmptyComponent={
            <Text className="text-text-secondary px-4 font-sans">
              No liked songs yet. Tap the heart on any track.
            </Text>
          }
          renderItem={({ item }) => {
            const isCurrent = currentTrack?.id === item.id && !!item.id;
            return (
              <Pressable
                onPress={() => playTrack(item)}
                className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover"
              >
                {item.artwork ? (
                  <Image
                    source={item.artwork}
                    className="w-11 h-11 rounded"
                    contentFit="cover"
                  />
                ) : (
                  <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
                    <Ionicons
                      name="musical-note"
                      size={16}
                      color={Colors.textMuted}
                    />
                  </View>
                )}
                <View className="flex-1">
                  <Text
                    numberOfLines={1}
                    className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
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
                {isCurrent ? (
                  <EqualizerBars size={14} playing={isPlaying} />
                ) : null}
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
