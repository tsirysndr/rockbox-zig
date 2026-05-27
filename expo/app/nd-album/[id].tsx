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
  const { playQueue, currentTrack, isPlaying } = usePlayer();
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
      {/* Sticky header */}
      <Animated.View
        style={{ opacity: headerBgOpacity }}
        className="absolute top-0 left-0 right-0 z-10 bg-bg"
      >
        <SafeAreaView edges={["top"]}>
          <View className="h-14 flex-row items-center px-4 gap-3">
            <Pressable hitSlop={10} onPress={() => router.back()}>
              <Ionicons name="chevron-back" size={26} color={Colors.textPrimary} />
            </Pressable>
            <Text numberOfLines={1} className="flex-1 text-text-primary text-[16px] font-display">
              {album?.name}
            </Text>
          </View>
        </SafeAreaView>
      </Animated.View>

      {/* Back button (always visible) */}
      <SafeAreaView edges={["top"]} className="absolute top-0 left-0 z-20">
        <Pressable hitSlop={10} onPress={() => router.back()} className="m-4">
          <View className="w-8 h-8 rounded-full bg-black/40 items-center justify-center">
            <Ionicons name="chevron-back" size={20} color="#FFFFFF" />
          </View>
        </Pressable>
      </SafeAreaView>

      <Animated.ScrollView
        showsVerticalScrollIndicator={false}
        scrollEventThrottle={16}
        onScroll={Animated.event(
          [{ nativeEvent: { contentOffset: { y: scrollY } } }],
          { useNativeDriver: true },
        )}
      >
        {/* Hero */}
        <Animated.View
          style={{ transform: [{ scale: artScale }], opacity: artOpacity }}
          className="items-center pt-16 pb-6"
        >
          {artSrc ? (
            <View style={{ width: ART_SIZE, height: ART_SIZE, borderRadius: 12, overflow: "hidden" }}>
              <Image
                source={{ uri: artSrc }}
                className="w-full h-full"
                contentFit="cover"
              />
            </View>
          ) : (
            <View
              style={{ width: ART_SIZE, height: ART_SIZE, borderRadius: 12, overflow: "hidden" }}
            >
              <LinearGradient
                colors={gradients}
                start={{ x: 0, y: 0 }}
                end={{ x: 1, y: 1 }}
                style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
              >
                <View style={{ opacity: 0.4 }}>
                  <Ionicons name="disc" size={88} color={gradientIconColor(gradients)} />
                </View>
              </LinearGradient>
            </View>
          )}
        </Animated.View>

        {/* Metadata */}
        <View className="px-5 pb-4">
          <Text className="text-text-primary text-[22px] font-display-extra" numberOfLines={2}>
            {album?.name ?? "Loading…"}
          </Text>
          <Pressable onPress={() => album && router.push(`/nd-artist/${album.artistId}` as any)}>
            <Text className="text-text-secondary text-[15px] mt-1 font-sans">
              {album?.artist}{album?.year ? ` · ${album.year}` : ""}
            </Text>
          </Pressable>
          <Text className="text-text-muted text-[13px] mt-0.5 font-sans">
            {songs.length} {songs.length === 1 ? "song" : "songs"}
          </Text>

          {/* Actions */}
          <View className="flex-row items-center gap-4 mt-5">
            <Pressable
              hitSlop={6}
              onPress={() => playQueue(tracks, { shuffle: true })}
              disabled={tracks.length === 0}
            >
              <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
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
        </View>

        {/* Track list */}
        <View style={{ paddingBottom: bottomPad }}>
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
                    className="flex-row items-center px-5 py-3 gap-4 active:bg-bg-hover"
                  >
                    <View className="w-6 items-end">
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
                        className={`text-[15px] font-semibold font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
                      >
                        {song.title}
                      </Text>
                      <Text numberOfLines={1} className="text-text-secondary text-[13px] font-sans">
                        {song.artist}
                      </Text>
                    </View>
                    <Pressable
                      hitSlop={8}
                      onPress={() =>
                        isStarred
                          ? unstarMut.mutate(song.id)
                          : starMut.mutate(song.id)
                      }
                    >
                      <Ionicons
                        name={isStarred ? "heart" : "heart-outline"}
                        size={18}
                        color={isStarred ? "#FFFFFF" : Colors.textMuted}
                      />
                    </Pressable>
                    <Text className="text-text-muted text-[13px] font-mono">
                      {formatDuration(song.duration)}
                    </Text>
                    {track && <TrackMenuButton track={track} />}
                  </Pressable>
                );
              })}
        </View>
      </Animated.ScrollView>
    </View>
  );
}
