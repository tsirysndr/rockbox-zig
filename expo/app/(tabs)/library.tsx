import { Ionicons } from "@expo/vector-icons";
import { useQueryClient } from "@tanstack/react-query";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
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

import { ArtistPlaceholder } from "@/components/artist-placeholder";
import { NotConnectedState } from "@/components/empty-state";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { EqualizerBars } from "@/components/equalizer-bars";
import { PlaylistCover, heroGradientColors, gradientIconColor } from "@/components/playlist-cover";
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
import { coverArtUrl } from "@/lib/navidrome-client";
import {
  useNdAlbums,
  useNdArtists,
  useNdSongs,
  useNdPlaylists,
  useNdStarred,
  useNdStarredIds,
  useNdStar,
  useNdUnstar,
} from "@/lib/navidrome-source";
import { useNdActiveServer } from "@/lib/navidrome-store";
import { usePlayer } from "@/lib/player-context";
import type { LibrarySection, Track } from "@/lib/types";

type NdSection = "nd-albums" | "nd-artists" | "nd-songs" | "nd-liked" | "nd-playlists";

const LOCAL_SECTIONS: { id: LibrarySection; label: string }[] = [
  { id: "playlists", label: "Playlists" },
  { id: "songs", label: "Songs" },
  { id: "albums", label: "Albums" },
  { id: "artists", label: "Artists" },
  { id: "liked", label: "Liked" },
];

const ND_SECTIONS: { id: NdSection; label: string }[] = [
  { id: "nd-albums", label: "Albums" },
  { id: "nd-artists", label: "Artists" },
  { id: "nd-songs", label: "Songs" },
  { id: "nd-liked", label: "Liked" },
  { id: "nd-playlists", label: "Playlists" },
];

type ActiveSection = LibrarySection | "navidrome";

export default function LibraryScreen() {
  const [section, setSection] = useState<ActiveSection>("playlists");
  const [ndSection, setNdSection] = useState<NdSection>("nd-albums");
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const { liked, userPlaylists, playTrack, playQueue, currentTrack, isPlaying } =
    usePlayer();
  const isConnected = useIsConnected();
  const ndServer = useNdActiveServer();
  const qc = useQueryClient();

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    RockboxClient.rescanLibrary();
    await qc.invalidateQueries({
      predicate: (q) => {
        const k = q.queryKey;
        if (!Array.isArray(k) || k.length < 2) return false;
        const type = k[1];
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

  // Local library data
  const { data: tracks } = useLibraryTracks();
  const { data: albums } = useLibraryAlbums();
  const { data: artists } = useLibraryArtists();
  const { data: playlists } = useLibraryPlaylists();
  const { data: likedTracks } = useLibraryLikedTracks();

  // ND data (only fetched when ND section is active)
  const { data: ndAlbums = [], isLoading: ndAlbumsLoading } = useNdAlbums();
  const { data: ndArtists = [], isLoading: ndArtistsLoading } = useNdArtists();
  const { data: ndSongs = [], isLoading: ndSongsLoading } = useNdSongs();
  const { data: ndPlaylists = [], isLoading: ndPlaylistsLoading } = useNdPlaylists();
  const { data: ndStarred = [], isLoading: ndStarredLoading } = useNdStarred();
  const ndStarredIds = useNdStarredIds();
  const starMut = useNdStar();
  const unstarMut = useNdUnstar();

  const q = searchQuery.trim().toLowerCase();

  const allPlaylists = useMemo(
    () => [...userPlaylists, ...playlists],
    [userPlaylists, playlists],
  );
  const filteredPlaylists = useMemo(
    () => (q ? allPlaylists.filter((p) => p.name.toLowerCase().includes(q)) : allPlaylists),
    [q, allPlaylists],
  );
  const filteredSongs = useMemo(
    () =>
      q
        ? tracks.filter(
            (t) => t.title.toLowerCase().includes(q) || t.artist.toLowerCase().includes(q),
          )
        : tracks,
    [q, tracks],
  );
  const filteredAlbums = useMemo(
    () =>
      q
        ? albums.filter(
            (a) => a.title.toLowerCase().includes(q) || a.artist.toLowerCase().includes(q),
          )
        : albums,
    [q, albums],
  );
  const filteredArtists = useMemo(
    () => (q ? artists.filter((a) => a.name.toLowerCase().includes(q)) : artists),
    [q, artists],
  );
  const filteredLiked = useMemo(
    () =>
      q
        ? likedTracks.filter(
            (t) => t.title.toLowerCase().includes(q) || t.artist.toLowerCase().includes(q),
          )
        : likedTracks,
    [q, likedTracks],
  );

  // ND search filters
  const filteredNdAlbums = useMemo(
    () => (q ? ndAlbums.filter((a) => a.name.toLowerCase().includes(q) || a.artist.toLowerCase().includes(q)) : ndAlbums),
    [q, ndAlbums],
  );
  const filteredNdArtists = useMemo(
    () => (q ? ndArtists.filter((a) => a.name.toLowerCase().includes(q)) : ndArtists),
    [q, ndArtists],
  );
  const ndSongsSorted = useMemo(
    () => [...ndSongs].sort((a, b) => a.title.localeCompare(b.title)),
    [ndSongs],
  );
  const filteredNdSongs = useMemo(
    () =>
      q
        ? ndSongsSorted.filter(
            (s) => s.title.toLowerCase().includes(q) || s.artist.toLowerCase().includes(q),
          )
        : ndSongsSorted,
    [q, ndSongsSorted],
  );
  const filteredNdStarred = useMemo(
    () =>
      q
        ? ndStarred.filter(
            (s) => s.title.toLowerCase().includes(q) || s.artist.toLowerCase().includes(q),
          )
        : ndStarred,
    [q, ndStarred],
  );
  const filteredNdPlaylists = useMemo(
    () => (q ? ndPlaylists.filter((p) => p.name.toLowerCase().includes(q)) : ndPlaylists),
    [q, ndPlaylists],
  );

  const topSections = [
    ...LOCAL_SECTIONS,
    ...(ndServer ? [{ id: "navidrome" as ActiveSection, label: "Navidrome" }] : []),
  ];

  // ── Render ─────────────────────────────────────────────────────────────────

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
                <Ionicons name="close-circle" size={16} color={Colors.textMuted} />
              </Pressable>
            ) : null}
          </View>
        </View>
      ) : null}

      {/* Top-level section pills */}
      <View className="h-12 mb-1">
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={{ paddingHorizontal: 16, alignItems: "center", gap: 8 }}
        >
          {topSections.map((s) => {
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

      {/* ND sub-section pills */}
      {section === "navidrome" ? (
        <View className="h-10 mb-1">
          <ScrollView
            horizontal
            showsHorizontalScrollIndicator={false}
            contentContainerStyle={{ paddingHorizontal: 16, alignItems: "center", gap: 6 }}
          >
            {ND_SECTIONS.map((s) => {
              const active = s.id === ndSection;
              return (
                <Pressable
                  key={s.id}
                  onPress={() => setNdSection(s.id)}
                  className={`h-7 px-3 rounded-full items-center justify-center ${active ? "bg-accent/80" : "bg-bg-elevated"}`}
                >
                  <Text className={`text-[12px] font-sans ${active ? "text-white font-semibold" : "text-text-secondary"}`}>
                    {s.label}
                  </Text>
                </Pressable>
              );
            })}
          </ScrollView>
        </View>
      ) : null}

      {/* ── Content ─────────────────────────────────────────────────────────── */}

      {!isConnected && section !== "navidrome" ? (
        <NotConnectedState />
      ) : section === "playlists" ? (
        <FlatList
          key="list-playlists"
          data={filteredPlaylists}
          keyExtractor={(p) => p.id}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={Colors.accent} colors={[Colors.accent]} />
          }
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: bottomPad, gap: 8 }}
          renderItem={({ item }) => (
            <Pressable
              onPress={() => router.push(`/playlist/${item.id}`)}
              className="flex-row items-center gap-3 py-1.5 active:opacity-80"
            >
              <PlaylistCover artwork={item.artwork} seed={item.id || item.name} size={60} rounded="md" iconSize={24} />
              <View className="flex-1">
                <Text numberOfLines={1} className="text-text-primary text-[15px] font-semibold font-sans">
                  {item.name}
                </Text>
                <Text numberOfLines={1} className="text-text-secondary text-[13px] mt-0.5 font-sans">
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
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={Colors.accent} colors={[Colors.accent]} />
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
                  <Image source={item.artwork} className="w-12 h-12 rounded" contentFit="cover" />
                ) : (
                  <View className="w-12 h-12 bg-bg-card rounded items-center justify-center">
                    <Ionicons name="musical-note" size={18} color={Colors.textMuted} />
                  </View>
                )}
                <View className="flex-1">
                  <Text numberOfLines={1} className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}>
                    {item.title}
                  </Text>
                  <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">
                    {item.artist}
                  </Text>
                </View>
                <Text className="text-text-muted text-xs font-mono">{formatDuration(item.duration)}</Text>
                {isCurrent ? <EqualizerBars size={14} playing={isPlaying} /> : null}
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
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={Colors.accent} colors={[Colors.accent]} />
          }
          contentContainerStyle={{ paddingBottom: bottomPad, gap: 16 }}
          renderItem={({ item }) => (
            <Pressable onPress={() => router.push(`/album/${item.id}`)} className="flex-1 max-w-[48%] active:opacity-80">
              {item.artwork ? (
                <Image source={item.artwork} className="w-full aspect-square rounded-md" contentFit="cover" />
              ) : (
                <View className="w-full aspect-square rounded-md overflow-hidden">
                  <LinearGradient colors={heroGradientColors(item.title)} start={{ x: 0, y: 0 }} end={{ x: 1, y: 1 }} style={{ flex: 1, alignItems: "center", justifyContent: "center" }}>
                    <View style={{ opacity: 0.4 }}>
                      <Ionicons name="disc" size={88} color={gradientIconColor(heroGradientColors(item.title))} />
                    </View>
                  </LinearGradient>
                </View>
              )}
              <Text numberOfLines={1} className="text-text-primary text-[13px] font-semibold mt-1.5 font-sans">{item.title}</Text>
              <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">{item.artist}</Text>
            </Pressable>
          )}
        />
      ) : section === "artists" ? (
        <FlatList
          key="list-artists"
          data={filteredArtists}
          keyExtractor={(a) => a.id}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={Colors.accent} colors={[Colors.accent]} />
          }
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: bottomPad, gap: 8 }}
          renderItem={({ item }) => (
            <Pressable onPress={() => router.push(`/artist/${item.id}`)} className="flex-row items-center gap-3.5 py-1.5 active:opacity-80">
              {item.image ? (
                <Image source={item.image} className="w-14 h-14 rounded-full" contentFit="cover" />
              ) : (
                <View className="w-14 h-14 rounded-full bg-bg-card items-center justify-center">
                  <ArtistPlaceholder size={24} />
                </View>
              )}
              <View className="flex-1">
                <Text className="text-text-primary text-[15px] font-semibold font-sans">{item.name}</Text>
                <Text className="text-text-secondary text-[13px] mt-0.5 font-sans">Artist</Text>
              </View>
            </Pressable>
          )}
        />
      ) : section === "liked" ? (
        <FlatList
          key="list-liked"
          data={filteredLiked}
          keyExtractor={(t) => t.id}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor={Colors.accent} colors={[Colors.accent]} />
          }
          ListHeaderComponent={
            <View className="px-4 pt-1 pb-4">
              <View className="flex-row items-center gap-3.5">
                <View className="w-20 h-20 rounded-md bg-accent items-center justify-center">
                  <Ionicons name="heart" size={36} color="#FFFFFF" />
                </View>
                <View className="flex-1">
                  <Text className="text-text-primary text-[22px] font-display-extra">Liked Songs</Text>
                  <Text className="text-text-secondary text-[13px] mt-1 font-sans">{likedTracks.length} liked tracks</Text>
                </View>
              </View>
              <View className="flex-row items-center gap-4 mt-4">
                <Pressable hitSlop={6} onPress={() => playQueue(likedTracks, { shuffle: true })} disabled={likedTracks.length === 0} className="active:opacity-70 disabled:opacity-40">
                  <Ionicons name="shuffle" size={26} color={likedTracks.length === 0 ? Colors.textMuted : Colors.textPrimary} />
                </Pressable>
                <View className="flex-1" />
                <Pressable onPress={() => playQueue(likedTracks)} disabled={likedTracks.length === 0} className="w-14 h-14 rounded-full items-center justify-center bg-accent active:opacity-85 disabled:opacity-40" style={{ shadowColor: Colors.accent, shadowOpacity: 0.5, shadowRadius: 14, shadowOffset: { width: 0, height: 6 } }}>
                  <Ionicons name="play" size={26} color="#FFFFFF" style={{ marginLeft: 3 }} />
                </Pressable>
              </View>
            </View>
          }
          ListEmptyComponent={<Text className="text-text-secondary px-4 font-sans">No liked songs yet. Tap the heart on any track.</Text>}
          renderItem={({ item }) => {
            const isCurrent = currentTrack?.id === item.id && !!item.id;
            return (
              <Pressable onPress={() => playTrack(item)} className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover">
                {item.artwork ? (
                  <Image source={item.artwork} className="w-11 h-11 rounded" contentFit="cover" />
                ) : (
                  <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
                    <Ionicons name="musical-note" size={16} color={Colors.textMuted} />
                  </View>
                )}
                <View className="flex-1">
                  <Text numberOfLines={1} className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}>{item.title}</Text>
                  <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">{item.artist}</Text>
                </View>
                {isCurrent ? <EqualizerBars size={14} playing={isPlaying} /> : null}
                <TrackMenuButton track={item} />
              </Pressable>
            );
          }}
        />
      ) : section === "navidrome" ? (
        <NdSectionContent
          ndSection={ndSection}
          ndAlbums={filteredNdAlbums}
          ndArtists={filteredNdArtists}
          ndSongs={filteredNdSongs}
          ndStarred={filteredNdStarred}
          ndPlaylists={filteredNdPlaylists}
          ndAlbumsLoading={ndAlbumsLoading}
          ndArtistsLoading={ndArtistsLoading}
          ndSongsLoading={ndSongsLoading}
          ndStarredLoading={ndStarredLoading}
          ndPlaylistsLoading={ndPlaylistsLoading}
          ndStarredIds={ndStarredIds}
          starMut={starMut}
          unstarMut={unstarMut}
          ndServer={ndServer}
          currentTrack={currentTrack}
          isPlaying={isPlaying}
          playQueue={playQueue}
          bottomPad={bottomPad}
          refreshing={refreshing}
          onRefresh={onRefresh}
        />
      ) : null}

      {/* Create playlist modal */}
      <Modal visible={createOpen} transparent animationType="fade" onRequestClose={() => setCreateOpen(false)}>
        <Pressable className="flex-1 bg-black/60" onPress={() => setCreateOpen(false)}>
          <Pressable onPress={(e) => e.stopPropagation()} className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7">
            <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />
            <Text className="text-text-primary text-base font-bold text-center py-2 font-sans">Create new</Text>
            {[
              { icon: "musical-notes-outline", label: "Playlist", desc: "Build a custom mix", href: "/playlist/new" },
              { icon: "list-outline", label: "Smart playlist", desc: "Auto-updates from rules", href: "/playlist/new?mode=smart" },
            ].map((item) => (
              <Pressable
                key={item.label}
                onPress={() => { setCreateOpen(false); setTimeout(() => router.push(item.href as any), 50); }}
                android_ripple={{ color: Colors.bgHover }}
                className="flex-row items-center px-5 py-3.5 gap-4 active:bg-bg-hover"
              >
                <View className="w-11 h-11 rounded-full bg-bg-card items-center justify-center">
                  <Ionicons name={item.icon as any} size={22} color={Colors.textPrimary} />
                </View>
                <View className="flex-1">
                  <Text className="text-text-primary text-[15px] font-semibold font-sans">{item.label}</Text>
                  <Text className="text-text-secondary text-xs mt-0.5 font-sans">{item.desc}</Text>
                </View>
              </Pressable>
            ))}
          </Pressable>
        </Pressable>
      </Modal>
    </SafeAreaView>
  );
}

// ── Navidrome section content ─────────────────────────────────────────────────

type NdSectionContentProps = {
  ndSection: NdSection;
  ndAlbums: import("@/lib/navidrome-client").NdAlbum[];
  ndArtists: import("@/lib/navidrome-client").NdArtist[];
  ndSongs: import("@/lib/navidrome-client").NdSong[];
  ndStarred: import("@/lib/navidrome-client").NdSong[];
  ndPlaylists: import("@/lib/navidrome-client").NdPlaylist[];
  ndAlbumsLoading: boolean;
  ndArtistsLoading: boolean;
  ndSongsLoading: boolean;
  ndStarredLoading: boolean;
  ndPlaylistsLoading: boolean;
  ndStarredIds: Set<string>;
  starMut: ReturnType<typeof useNdStar>;
  unstarMut: ReturnType<typeof useNdUnstar>;
  ndServer: import("@/lib/navidrome-store").NdServer | null;
  currentTrack: Track | undefined;
  isPlaying: boolean;
  playQueue: (tracks: Track[], opts?: { startIdx?: number; shuffle?: boolean }) => void;
  bottomPad: number;
  refreshing: boolean;
  onRefresh: () => void;
};

function NdSectionContent({
  ndSection,
  ndAlbums,
  ndArtists,
  ndSongs,
  ndStarred,
  ndPlaylists,
  ndAlbumsLoading,
  ndArtistsLoading,
  ndSongsLoading,
  ndStarredLoading,
  ndPlaylistsLoading,
  ndStarredIds,
  starMut,
  unstarMut,
  ndServer,
  currentTrack,
  isPlaying,
  playQueue,
  bottomPad,
  refreshing,
  onRefresh,
}: NdSectionContentProps) {
  const rc = (
    <RefreshControl
      refreshing={refreshing}
      onRefresh={onRefresh}
      tintColor={Colors.accent}
      colors={[Colors.accent]}
    />
  );

  function ndSongToTrack(song: import("@/lib/navidrome-client").NdSong): Track {
    return {
      id: song.id,
      path: song.streamUrl,
      title: song.title,
      artist: song.artist,
      artistId: song.artistId,
      album: song.album,
      albumId: song.albumId,
      duration: song.duration,
      artwork: song.coverArt && ndServer
        ? coverArtUrl(ndServer.baseUrl, ndServer.user, ndServer.password, song.coverArt, 150)
        : undefined,
    };
  }

  // ── Albums ────────────────────────────────────────────────────────────────

  if (ndSection === "nd-albums") {
    return (
      <FlatList
        key="nd-albums"
        data={ndAlbums}
        keyExtractor={(a) => a.id}
        numColumns={2}
        columnWrapperStyle={{ gap: 12, paddingHorizontal: 16 }}
        refreshControl={rc}
        contentContainerStyle={{ paddingBottom: bottomPad, gap: 16 }}
        ListEmptyComponent={
          ndAlbumsLoading ? (
            <View className="flex-row flex-wrap gap-3 px-4">
              {Array.from({ length: 6 }, (_, i) => (
                <View key={i} className="flex-1 max-w-[48%]">
                  <View className="w-full aspect-square rounded-md bg-bg-card" />
                  <View className="w-3/4 h-3.5 rounded bg-bg-card mt-1.5" />
                  <View className="w-1/2 h-3 rounded bg-bg-card mt-1" />
                </View>
              ))}
            </View>
          ) : !ndServer ? (
            <View className="px-4 py-8 items-center gap-3">
              <Ionicons name="musical-notes-outline" size={40} color={Colors.textMuted} />
              <Text className="text-text-secondary text-[14px] font-sans text-center">
                Connect a Navidrome server in Settings → Navidrome / Subsonic
              </Text>
              <Pressable onPress={() => router.push("/settings/navidrome" as any)} className="mt-2 px-4 py-2 rounded-full bg-accent active:opacity-80">
                <Text className="text-white text-[13px] font-display">Connect</Text>
              </Pressable>
            </View>
          ) : null
        }
        renderItem={({ item }) => {
          const artSrc = item.coverArt && ndServer
            ? coverArtUrl(ndServer.baseUrl, ndServer.user, ndServer.password, item.coverArt, 300)
            : null;
          return (
            <Pressable onPress={() => router.push(`/nd-album/${item.id}` as any)} className="flex-1 max-w-[48%] active:opacity-80">
              {artSrc ? (
                <Image source={artSrc} className="w-full aspect-square rounded-md" contentFit="cover" />
              ) : (
                <View className="w-full aspect-square rounded-md overflow-hidden">
                  <LinearGradient colors={heroGradientColors(item.name)} start={{ x: 0, y: 0 }} end={{ x: 1, y: 1 }} style={{ flex: 1, alignItems: "center", justifyContent: "center" }}>
                    <View style={{ opacity: 0.4 }}>
                      <Ionicons name="disc" size={64} color={gradientIconColor(heroGradientColors(item.name))} />
                    </View>
                  </LinearGradient>
                </View>
              )}
              <Text numberOfLines={1} className="text-text-primary text-[13px] font-semibold mt-1.5 font-sans">{item.name}</Text>
              <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">{item.artist}</Text>
            </Pressable>
          );
        }}
      />
    );
  }

  // ── Artists ───────────────────────────────────────────────────────────────

  if (ndSection === "nd-artists") {
    return (
      <FlatList
        key="nd-artists"
        data={ndArtists}
        keyExtractor={(a) => a.id}
        refreshControl={rc}
        contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: bottomPad, gap: 8 }}
        ListEmptyComponent={
          ndArtistsLoading ? (
            <View className="gap-3">
              {Array.from({ length: 8 }, (_, i) => (
                <View key={i} className="flex-row items-center gap-3.5 py-1.5">
                  <View className="w-14 h-14 rounded-full bg-bg-card" />
                  <View className="gap-1.5">
                    <View className="w-32 h-3.5 rounded bg-bg-card" />
                    <View className="w-20 h-3 rounded bg-bg-card" />
                  </View>
                </View>
              ))}
            </View>
          ) : null
        }
        renderItem={({ item }) => {
          const artSrc = item.coverArt && ndServer
            ? coverArtUrl(ndServer.baseUrl, ndServer.user, ndServer.password, item.coverArt, 150)
            : null;
          return (
            <Pressable onPress={() => router.push(`/nd-artist/${item.id}` as any)} className="flex-row items-center gap-3.5 py-1.5 active:opacity-80">
              {artSrc ? (
                <Image source={artSrc} className="w-14 h-14 rounded-full" contentFit="cover" />
              ) : (
                <View className="w-14 h-14 rounded-full bg-bg-card items-center justify-center">
                  <ArtistPlaceholder size={24} />
                </View>
              )}
              <View className="flex-1">
                <Text className="text-text-primary text-[15px] font-semibold font-sans">{item.name}</Text>
                <Text className="text-text-secondary text-[13px] mt-0.5 font-sans">
                  {item.albumCount} {item.albumCount === 1 ? "album" : "albums"}
                </Text>
              </View>
            </Pressable>
          );
        }}
      />
    );
  }

  // ── Songs ─────────────────────────────────────────────────────────────────

  if (ndSection === "nd-songs") {
    const tracks = ndSongs.map(ndSongToTrack);
    return (
      <FlatList
        key="nd-songs"
        data={ndSongs}
        keyExtractor={(s) => s.id}
        refreshControl={rc}
        contentContainerStyle={{ paddingBottom: bottomPad }}
        ListEmptyComponent={
          ndSongsLoading ? (
            <View>
              {Array.from({ length: 10 }, (_, i) => (
                <View key={i} className="flex-row items-center gap-3 px-4 py-2.5">
                  <View className="w-12 h-12 rounded bg-bg-card" />
                  <View className="flex-1 gap-1.5">
                    <View className="w-3/4 h-3.5 rounded bg-bg-card" />
                    <View className="w-1/2 h-3 rounded bg-bg-card" />
                  </View>
                </View>
              ))}
            </View>
          ) : null
        }
        renderItem={({ item, index }) => {
          const track = tracks[index];
          const isCurrent = currentTrack?.id === item.id;
          const isStarred = ndStarredIds.has(item.id);
          const artSrc = item.coverArt && ndServer
            ? coverArtUrl(ndServer.baseUrl, ndServer.user, ndServer.password, item.coverArt, 150)
            : null;
          return (
            <Pressable
              onPress={() => track && playQueue(tracks, { startIdx: index })}
              className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover"
            >
              <Text className="text-text-muted text-[13px] font-mono w-6 text-right">{index + 1}</Text>
              {artSrc ? (
                <Image source={artSrc} className="w-11 h-11 rounded" contentFit="cover" />
              ) : (
                <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
                  <Ionicons name="musical-note" size={16} color={Colors.textMuted} />
                </View>
              )}
              <View className="flex-1">
                <Text numberOfLines={1} className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}>{item.title}</Text>
                <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">{item.artist}</Text>
              </View>
              {isCurrent ? <EqualizerBars size={14} playing={isPlaying} /> : null}
              <Pressable hitSlop={8} onPress={() => isStarred ? unstarMut.mutate(item.id) : starMut.mutate(item.id)}>
                <Ionicons name={isStarred ? "heart" : "heart-outline"} size={18} color={isStarred ? "#FFFFFF" : Colors.textMuted} />
              </Pressable>
              <Text className="text-text-muted text-xs font-mono">{formatDuration(item.duration)}</Text>
            </Pressable>
          );
        }}
      />
    );
  }

  // ── Liked (starred) ───────────────────────────────────────────────────────

  if (ndSection === "nd-liked") {
    const tracks = ndStarred.map(ndSongToTrack);
    return (
      <FlatList
        key="nd-liked"
        data={ndStarred}
        keyExtractor={(s) => s.id}
        refreshControl={rc}
        ListHeaderComponent={
          <View className="px-4 pt-1 pb-4">
            <View className="flex-row items-center gap-3.5">
              <View className="w-20 h-20 rounded-md bg-accent items-center justify-center">
                <Ionicons name="heart" size={36} color="#FFFFFF" />
              </View>
              <View className="flex-1">
                <Text className="text-text-primary text-[22px] font-display-extra">Liked Songs</Text>
                <Text className="text-text-secondary text-[13px] mt-1 font-sans">{ndStarred.length} liked tracks</Text>
              </View>
            </View>
            <View className="flex-row items-center gap-4 mt-4">
              <Pressable hitSlop={6} onPress={() => playQueue(tracks, { shuffle: true })} disabled={tracks.length === 0}>
                <Ionicons name="shuffle" size={26} color={tracks.length === 0 ? Colors.textMuted : Colors.textPrimary} />
              </Pressable>
              <View className="flex-1" />
              <Pressable onPress={() => playQueue(tracks)} disabled={tracks.length === 0} className="w-14 h-14 rounded-full items-center justify-center bg-accent active:opacity-85 disabled:opacity-40" style={{ shadowColor: Colors.accent, shadowOpacity: 0.5, shadowRadius: 14, shadowOffset: { width: 0, height: 6 } }}>
                <Ionicons name="play" size={26} color="#FFFFFF" style={{ marginLeft: 3 }} />
              </Pressable>
            </View>
          </View>
        }
        contentContainerStyle={{ paddingBottom: bottomPad }}
        ListEmptyComponent={
          ndStarredLoading ? null : (
            <Text className="text-text-secondary px-4 font-sans">No liked songs yet.</Text>
          )
        }
        renderItem={({ item, index }) => {
          const track = tracks[index];
          const isCurrent = currentTrack?.id === item.id;
          const artSrc = item.coverArt && ndServer
            ? coverArtUrl(ndServer.baseUrl, ndServer.user, ndServer.password, item.coverArt, 150)
            : null;
          return (
            <Pressable onPress={() => track && playQueue(tracks, { startIdx: index })} className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover">
              {artSrc ? (
                <Image source={artSrc} className="w-11 h-11 rounded" contentFit="cover" />
              ) : (
                <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
                  <Ionicons name="musical-note" size={16} color={Colors.textMuted} />
                </View>
              )}
              <View className="flex-1">
                <Text numberOfLines={1} className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}>{item.title}</Text>
                <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">{item.artist}</Text>
              </View>
              {isCurrent ? <EqualizerBars size={14} playing={isPlaying} /> : null}
              <Pressable hitSlop={8} onPress={() => unstarMut.mutate(item.id)}>
                <Ionicons name="heart" size={18} color="#FFFFFF" />
              </Pressable>
              <Text className="text-text-muted text-xs font-mono">{formatDuration(item.duration)}</Text>
            </Pressable>
          );
        }}
      />
    );
  }

  // ── Playlists ─────────────────────────────────────────────────────────────

  return (
    <FlatList
      key="nd-playlists"
      data={ndPlaylists}
      keyExtractor={(p) => p.id}
      refreshControl={rc}
      contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: bottomPad, gap: 8 }}
      ListEmptyComponent={
        ndPlaylistsLoading ? (
          <View className="gap-3">
            {Array.from({ length: 5 }, (_, i) => (
              <View key={i} className="flex-row items-center gap-3 py-1.5">
                <View className="w-16 h-16 rounded-md bg-bg-card" />
                <View className="gap-1.5">
                  <View className="w-36 h-3.5 rounded bg-bg-card" />
                  <View className="w-20 h-3 rounded bg-bg-card" />
                </View>
              </View>
            ))}
          </View>
        ) : null
      }
      renderItem={({ item }) => {
        const artSrc = item.coverArt && ndServer
          ? coverArtUrl(ndServer.baseUrl, ndServer.user, ndServer.password, item.coverArt, 150)
          : null;
        return (
          <Pressable onPress={() => router.push(`/nd-playlist/${item.id}` as any)} className="flex-row items-center gap-3 py-1.5 active:opacity-80">
            {artSrc ? (
              <Image source={artSrc} className="w-16 h-16 rounded-md" contentFit="cover" />
            ) : (
              <View className="w-16 h-16 rounded-md bg-bg-card items-center justify-center">
                <Ionicons name="musical-notes" size={24} color={Colors.textMuted} />
              </View>
            )}
            <View className="flex-1">
              <Text numberOfLines={1} className="text-text-primary text-[15px] font-semibold font-sans">{item.name}</Text>
              <Text numberOfLines={1} className="text-text-secondary text-[13px] mt-0.5 font-sans">
                Playlist{item.songCount > 0 ? ` • ${item.songCount} tracks` : ""}
              </Text>
            </View>
          </Pressable>
        );
      }}
    />
  );
}
