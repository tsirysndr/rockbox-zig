import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { router, useLocalSearchParams } from "expo-router";
import { useMemo, useState } from "react";
import {
  Animated,
  Dimensions,
  Pressable,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { ActionSheet, type ActionItem } from "@/components/action-sheet";
import { EqualizerBars } from "@/components/equalizer-bars";
import { heroGradientColors, gradientIconColor } from "@/components/playlist-cover";
import { TrackMenuButton } from "@/components/track-menu-button";
import { Colors } from "@/constants/theme";
import { formatDuration } from "@/lib/mock-data";
import { coverArtUrl } from "@/lib/navidrome-client";
import { useNdAlbumDetail, useNdStarredIds, useNdStar, useNdUnstar } from "@/lib/navidrome-source";
import { useNdActiveServer } from "@/lib/navidrome-store";
import { usePlayer } from "@/lib/player-context";
import type { Track } from "@/lib/types";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

const { width } = Dimensions.get("window");
const ART_SIZE = Math.min(width * 0.62, 280);
const HEADER_HEIGHT = 56;

function ndSongToTrack(song: import("@/lib/navidrome-client").NdSong, baseUrl: string, user: string, password: string): Track {
  return {
    id: song.id,
    path: song.streamUrl,
    title: song.title,
    artist: song.artist,
    artistId: song.artistId,
    album: song.album,
    albumId: song.albumId,
    duration: song.duration,
    artwork: song.coverArt ? coverArtUrl(baseUrl, user, password, song.coverArt, 300) : undefined,
    trackNumber: song.track,
  };
}

export default function NdAlbumScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const nd = useNdActiveServer();
  const { data: detail, isLoading } = useNdAlbumDetail(id ?? "");
  const starredIds = useNdStarredIds();
  const starMut = useNdStar();
  const unstarMut = useNdUnstar();
  const { playQueue, playLast, currentTrack, isPlaying } = usePlayer();
  const [menuOpen, setMenuOpen] = useState(false);
  const bottomPad = useBottomSpacing(24);

  const scrollY = useMemo(() => new Animated.Value(0), []);
  const artScale = scrollY.interpolate({
    inputRange: [-100, 0, 200],
    outputRange: [1.1, 1, 0.85],
    extrapolate: "clamp",
  });
  const artOpacity = scrollY.interpolate({
    inputRange: [0, 200, 320],
    outputRange: [1, 1, 0],
    extrapolate: "clamp",
  });
  const headerBgOpacity = scrollY.interpolate({
    inputRange: [0, 200, 280],
    outputRange: [0, 0, 1],
    extrapolate: "clamp",
  });
  const titleOpacity = scrollY.interpolate({
    inputRange: [0, 240, 320],
    outputRange: [0, 0, 1],
    extrapolate: "clamp",
  });

  const album = detail?.album;
  const songs = detail?.songs ?? [];

  const artSrc = album?.coverArt && nd
    ? coverArtUrl(nd.baseUrl, nd.user, nd.password, album.coverArt, 500)
    : null;

  const sortedSongs = useMemo(
    () => [...songs].sort((a, b) => (a.track ?? 0) - (b.track ?? 0)),
    [songs],
  );

  const tracks: Track[] = useMemo(() => {
    if (!nd) return [];
    return sortedSongs.map((s) => ndSongToTrack(s, nd.baseUrl, nd.user, nd.password));
  }, [sortedSongs, nd]);

  if (!isLoading && !album) {
    return (
      <SafeAreaView className="flex-1 bg-bg">
        <View className="flex-1 items-center justify-center">
          <Text className="text-text-secondary font-sans">Album not found</Text>
        </View>
      </SafeAreaView>
    );
  }

  const gradients = heroGradientColors(album?.name ?? "");

  return (
    <View className="flex-1 bg-bg">
      <Animated.ScrollView
        showsVerticalScrollIndicator={false}
        scrollEventThrottle={16}
        onScroll={Animated.event(
          [{ nativeEvent: { contentOffset: { y: scrollY } } }],
          { useNativeDriver: true },
        )}
        contentContainerStyle={{ paddingBottom: bottomPad }}
      >
        {/* Hero band: blurred album art behind, sharp art front-and-center */}
        <View
          className="items-center overflow-hidden pb-4"
          style={{ paddingTop: HEADER_HEIGHT + 16 }}
        >
          {artSrc ? (
            <Image
              source={{ uri: artSrc }}
              className="absolute inset-0"
              contentFit="cover"
              blurRadius={40}
            />
          ) : (
            <LinearGradient
              colors={gradients}
              start={{ x: 0, y: 0 }}
              end={{ x: 1, y: 1 }}
              className="absolute inset-0"
            />
          )}
          <LinearGradient
            colors={["rgba(0,0,0,0.35)", "rgba(0,0,0,0.6)", "rgba(0,0,0,1)"]}
            locations={[0, 0.55, 1]}
            className="absolute inset-0"
          />
          <Animated.View
            style={{
              transform: [{ scale: artScale }],
              opacity: artOpacity,
              shadowColor: "#000",
              shadowOpacity: 0.6,
              shadowRadius: 20,
              shadowOffset: { width: 0, height: 12 },
            }}
          >
            {artSrc ? (
              <Image
                source={{ uri: artSrc }}
                className="rounded-lg"
                style={{ width: ART_SIZE, height: ART_SIZE }}
                contentFit="cover"
              />
            ) : (
              <View
                className="rounded-lg overflow-hidden"
                style={{ width: Math.round(ART_SIZE * 0.82), height: Math.round(ART_SIZE * 0.82) }}
              >
                <LinearGradient
                  colors={gradients}
                  start={{ x: 0, y: 0 }}
                  end={{ x: 1, y: 1 }}
                  style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
                >
                  <View style={{ opacity: 0.4 }}>
                    <Ionicons name="disc" size={Math.round(ART_SIZE * 0.82 * 0.55)} color={gradientIconColor(gradients)} />
                  </View>
                </LinearGradient>
              </View>
            )}
          </Animated.View>
        </View>

        {/* Title block */}
        <View className="px-5 mt-3.5">
          <Text className="text-text-primary text-[26px] font-display-extra" numberOfLines={2}>
            {album?.name ?? "Loading…"}
          </Text>
          <Pressable
            hitSlop={6}
            onPress={() => album && router.push(`/nd-artist/${album.artistId}` as any)}
            className="mt-1.5"
          >
            <Text className="text-text-primary text-sm font-semibold font-sans">
              {album?.artist}
            </Text>
          </Pressable>
          <Text className="text-text-secondary text-xs mt-1 font-sans">
            Album{album?.year ? ` • ${album.year}` : ""} • {songs.length}{" "}
            {songs.length === 1 ? "track" : "tracks"}
          </Text>
        </View>

        {/* Action row */}
        <View className="flex-row items-center px-5 mt-5 gap-4">
          <Pressable
            hitSlop={6}
            onPress={() => playQueue(tracks, { shuffle: true })}
            disabled={tracks.length === 0}
            className="w-11 h-11 rounded-full items-center justify-center"
          >
            <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
          </Pressable>
          <Pressable hitSlop={6} onPress={() => setMenuOpen(true)}>
            <Ionicons name="ellipsis-horizontal" size={26} color={Colors.textPrimary} />
          </Pressable>
          <View className="flex-1" />
          <Pressable
            onPress={() => playQueue(tracks)}
            disabled={tracks.length === 0}
            className="w-14 h-14 rounded-full items-center justify-center bg-accent active:opacity-85 disabled:opacity-40"
            style={{
              shadowColor: Colors.accent,
              shadowOpacity: 0.5,
              shadowRadius: 14,
              shadowOffset: { width: 0, height: 6 },
            }}
          >
            <Ionicons name="play" size={26} color="#FFFFFF" style={{ marginLeft: 3 }} />
          </Pressable>
        </View>

        {/* Track list */}
        <View className="mt-5">
          {isLoading
            ? Array.from({ length: 8 }, (_, i) => (
                <View key={i} className="flex-row items-center px-5 py-3 gap-4">
                  <View className="w-6 items-end">
                    <View className="w-4 h-3 rounded bg-bg-card" />
                  </View>
                  <View className="flex-1 gap-1.5">
                    <View className="w-3/4 h-3.5 rounded bg-bg-card" />
                    <View className="w-1/2 h-3 rounded bg-bg-card" />
                  </View>
                </View>
              ))
            : sortedSongs.map((song, idx) => {
                const track = tracks[idx];
                const isCurrent = currentTrack?.id === song.id;
                const isStarred = starredIds.has(song.id);
                return (
                  <Pressable
                    key={song.id}
                    onPress={() => track && playQueue(tracks, { startIdx: idx })}
                    className="flex-row items-center px-5 py-2.5 gap-3.5 active:bg-bg-hover"
                  >
                    <View className="w-[22px] items-center">
                      {isCurrent ? (
                        <EqualizerBars size={14} playing={isPlaying} />
                      ) : (
                        <Text className="text-text-muted text-[13px] font-mono">
                          {song.track ?? idx + 1}
                        </Text>
                      )}
                    </View>
                    <View className="flex-1">
                      <Text
                        numberOfLines={1}
                        className={`text-[15px] font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
                      >
                        {song.title}
                      </Text>
                      <Text numberOfLines={1} className="text-text-secondary text-xs mt-0.5 font-sans">
                        {song.artist}
                      </Text>
                    </View>
                    <Pressable
                      hitSlop={8}
                      onPress={() =>
                        isStarred ? unstarMut.mutate(song.id) : starMut.mutate(song.id)
                      }
                    >
                      <Ionicons
                        name={isStarred ? "heart" : "heart-outline"}
                        size={18}
                        color={isStarred ? "#FFFFFF" : Colors.textMuted}
                      />
                    </Pressable>
                    <Text className="text-text-muted text-xs font-mono">
                      {formatDuration(song.duration)}
                    </Text>
                    {track && <TrackMenuButton track={track} />}
                  </Pressable>
                );
              })}
        </View>
      </Animated.ScrollView>

      <ActionSheet
        visible={menuOpen}
        onClose={() => setMenuOpen(false)}
        header={{
          title: album?.name ?? "",
          subtitle: album?.artist,
          image: artSrc ?? undefined,
        }}
        actions={
          [
            {
              icon: "play-outline",
              label: "Play",
              onPress: () => { setMenuOpen(false); playQueue(tracks); },
            },
            {
              icon: "shuffle",
              label: "Shuffle Play",
              onPress: () => { setMenuOpen(false); playQueue(tracks, { shuffle: true }); },
            },
            {
              icon: "list-outline",
              label: "Add to Queue",
              onPress: () => { setMenuOpen(false); tracks.forEach((t) => playLast(t)); },
            },
          ] as ActionItem[]
        }
      />

      {/* Sticky header */}
      <SafeAreaView
        edges={["top"]}
        className="absolute top-0 left-0 right-0"
        pointerEvents="box-none"
      >
        <Animated.View
          pointerEvents="none"
          className="absolute top-0 left-0 right-0 bg-bg"
          style={{ height: HEADER_HEIGHT + 64, opacity: headerBgOpacity }}
        />
        <View
          className="flex-row items-center px-3"
          style={{ height: HEADER_HEIGHT }}
        >
          <Pressable
            hitSlop={10}
            onPress={() => router.back()}
            className="w-10 h-10 rounded-full items-center justify-center bg-black/35"
          >
            <Ionicons name="chevron-back" size={22} color={Colors.textPrimary} />
          </Pressable>
          <Animated.Text
            numberOfLines={1}
            className="flex-1 text-center text-text-primary text-[15px] font-bold px-3 font-sans"
            style={{ opacity: titleOpacity }}
          >
            {album?.name}
          </Animated.Text>
          <View className="w-10" />
        </View>
      </SafeAreaView>
    </View>
  );
}
