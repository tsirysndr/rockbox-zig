import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router, useLocalSearchParams } from "expo-router";
import { useMemo } from "react";
import { Pressable, ScrollView, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { EqualizerBars } from "@/components/equalizer-bars";
import { Colors } from "@/constants/theme";
import { formatDuration } from "@/lib/mock-data";
import { coverArtUrl } from "@/lib/navidrome-client";
import { useNdPlaylistDetail, useNdStarredIds, useNdStar, useNdUnstar } from "@/lib/navidrome-source";
import { useNdActiveServer } from "@/lib/navidrome-store";
import { usePlayer } from "@/lib/player-context";
import type { Track } from "@/lib/types";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

export default function NdPlaylistScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const nd = useNdActiveServer();
  const { data: detail, isLoading } = useNdPlaylistDetail(id ?? "");
  const starredIds = useNdStarredIds();
  const starMut = useNdStar();
  const unstarMut = useNdUnstar();
  const { playQueue, currentTrack, isPlaying } = usePlayer();
  const bottomPad = useBottomSpacing(24);

  const playlist = detail?.playlist;
  const songs = detail?.songs ?? [];

  const tracks: Track[] = useMemo(() => {
    if (!nd) return [];
    return songs.map((song) => ({
      id: song.id,
      path: song.streamUrl,
      title: song.title,
      artist: song.artist,
      artistId: song.artistId,
      album: song.album,
      albumId: song.albumId,
      duration: song.duration,
      artwork: song.coverArt
        ? coverArtUrl(nd.baseUrl, nd.user, nd.password, song.coverArt, 300)
        : undefined,
    }));
  }, [songs, nd]);

  const artSrc = playlist?.coverArt && nd
    ? coverArtUrl(nd.baseUrl, nd.user, nd.password, playlist.coverArt, 300)
    : null;

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <View className="flex-row items-center px-4 py-3 gap-3 border-b border-divider">
        <Pressable hitSlop={10} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={26} color={Colors.textPrimary} />
        </Pressable>
        <Text numberOfLines={1} className="flex-1 text-text-primary text-[18px] font-display-extra">
          {playlist?.name ?? "Playlist"}
        </Text>
      </View>

      <ScrollView contentContainerStyle={{ paddingBottom: bottomPad }}>
        {/* Header */}
        <View className="items-center px-5 pt-6 pb-5">
          {artSrc ? (
            <Image source={artSrc} className="w-36 h-36 rounded-xl" contentFit="cover" />
          ) : (
            <View className="w-36 h-36 rounded-xl bg-bg-card items-center justify-center">
              <Ionicons name="musical-notes" size={48} color={Colors.textMuted} />
            </View>
          )}
          <Text className="text-text-primary text-[20px] font-display-extra mt-3 text-center">
            {playlist?.name}
          </Text>
          {playlist?.comment ? (
            <Text className="text-text-secondary text-[13px] mt-1 text-center font-sans">
              {playlist.comment}
            </Text>
          ) : null}
          <Text className="text-text-muted text-[13px] mt-1 font-sans">
            {songs.length} {songs.length === 1 ? "song" : "songs"}
          </Text>

          {/* Actions */}
          <View className="flex-row items-center gap-5 mt-5">
            <Pressable
              hitSlop={6}
              onPress={() => playQueue(tracks, { shuffle: true })}
              disabled={tracks.length === 0}
            >
              <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
            </Pressable>
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
        {isLoading
          ? Array.from({ length: 6 }, (_, i) => (
              <View key={i} className="flex-row items-center px-4 py-2.5 gap-3">
                <View className="w-11 h-11 rounded bg-bg-card" />
                <View className="flex-1 gap-1.5">
                  <View className="w-3/4 h-3.5 rounded bg-bg-card" />
                  <View className="w-1/2 h-3 rounded bg-bg-card" />
                </View>
              </View>
            ))
          : songs.map((song, idx) => {
              const track = tracks[idx];
              const isCurrent = currentTrack?.id === song.id;
              const isStarred = starredIds.has(song.id);
              const artUrl = song.coverArt && nd
                ? coverArtUrl(nd.baseUrl, nd.user, nd.password, song.coverArt, 150)
                : null;
              return (
                <Pressable
                  key={song.id}
                  onPress={() => track && playQueue(tracks, { startIdx: idx })}
                  className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover"
                >
                  {artUrl ? (
                    <Image source={artUrl} className="w-11 h-11 rounded" contentFit="cover" />
                  ) : (
                    <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
                      <Ionicons name="musical-note" size={16} color={Colors.textMuted} />
                    </View>
                  )}
                  <View className="flex-1">
                    <Text
                      numberOfLines={1}
                      className={`text-sm font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
                    >
                      {song.title}
                    </Text>
                    <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">
                      {song.artist}
                    </Text>
                  </View>
                  {isCurrent ? <EqualizerBars size={14} playing={isPlaying} /> : null}
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
                </Pressable>
              );
            })}
      </ScrollView>
    </SafeAreaView>
  );
}
