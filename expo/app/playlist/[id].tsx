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
import { PlaylistCover, gradientColors } from "@/components/playlist-cover";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { TrackMenuButton } from "@/components/track-menu-button";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import { usePlaylistDetail } from "@/lib/library-source";
import { formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

const { width } = Dimensions.get("window");
// Compact hero — playlist screens get visited from rows where the user
// already saw the cover, so the title / actions / track list should sit
// higher than on album / artist screens.
const ART_SIZE = Math.min(width * 0.45, 190);
const HEADER_HEIGHT = 56;

export default function PlaylistScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const { playQueue, currentTrack, isPlaying, userPlaylists, playLast } =
    usePlayer();
  const detail = usePlaylistDetail(id ?? "");
  const playlist = id
    ? userPlaylists.find((p) => p.id === id) ?? detail.playlist
    : undefined;
  const tracks = detail.tracks;
  const [menuOpen, setMenuOpen] = useState(false);
  const isUserPlaylist = !!userPlaylists.find((p) => p.id === id);
  const bottomPad = useBottomSpacing(24);

  const totalDuration = tracks.reduce((s, t) => s + t.duration, 0);
  const totalMinutes = Math.round(totalDuration / 60);

  const scrollY = useMemo(() => new Animated.Value(0), []);
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

  if (!playlist) {
    return (
      <SafeAreaView className="flex-1 bg-bg">
        <View className="flex-1 items-center justify-center">
          <Text className="text-text-secondary">Playlist not found</Text>
        </View>
      </SafeAreaView>
    );
  }

  const onPlay = () => playQueue(tracks);
  const onShuffle = () => playQueue(tracks, { shuffle: true });

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
        {/* Hero band: blurred art behind, sharp art front-and-center */}
        <View
          className="items-center overflow-hidden pb-2"
          style={{ paddingTop: HEADER_HEIGHT + 8 }}
        >
          {playlist.artwork ? (
            <Image
              source={playlist.artwork}
              className="absolute inset-0"
              contentFit="cover"
              blurRadius={40}
            />
          ) : (
            <LinearGradient
              colors={gradientColors(playlist.id || playlist.name)}
              start={{ x: 0, y: 0 }}
              end={{ x: 1, y: 1 }}
              className="absolute inset-0"
            />
          )}
          <LinearGradient
            colors={[
              "rgba(0,0,0,0.35)",
              "rgba(0,0,0,0.6)",
              "rgba(0,0,0,1)",
            ]}
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
            <PlaylistCover
              artwork={playlist.artwork}
              seed={playlist.id || playlist.name}
              size={ART_SIZE}
              rounded="lg"
            />
          </Animated.View>
        </View>

        {/* Title block */}
        <View className="px-5 mt-2">
          <Text className="text-text-primary text-[26px] font-display-extra">
            {playlist.name}
          </Text>
          {playlist.description ? (
            <Text className="text-text-secondary text-[13px] mt-1.5 font-sans">
              {playlist.description}
            </Text>
          ) : null}
          <Text className="text-text-muted text-xs mt-2 font-sans">
            {[
              playlist.isSmart ? "Smart playlist" : "Playlist",
              tracks.length > 0 ? `${tracks.length} tracks` : null,
              tracks.length > 0 ? `${totalMinutes} min` : null,
            ]
              .filter(Boolean)
              .join(" • ")}
          </Text>
        </View>

        {/* Action row */}
        <View className="flex-row items-center px-5 mt-3 gap-4">
          <Pressable
            hitSlop={6}
            onPress={onShuffle}
            disabled={tracks.length === 0}
            className={`w-11 h-11 rounded-full items-center justify-center ${tracks.length === 0 ? "opacity-40" : ""}`}
          >
            <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
          </Pressable>
          <Pressable
            hitSlop={6}
            disabled={tracks.length === 0}
            className={tracks.length === 0 ? "opacity-40" : ""}
          >
            <Ionicons
              name="heart-outline"
              size={26}
              color={Colors.textPrimary}
            />
          </Pressable>
          <Pressable
            hitSlop={6}
            onPress={() => setMenuOpen(true)}
            disabled={tracks.length === 0}
            className={tracks.length === 0 ? "opacity-40" : ""}
          >
            <Ionicons
              name="ellipsis-horizontal"
              size={26}
              color={Colors.textPrimary}
            />
          </Pressable>
          <View className="flex-1" />
          <Pressable
            onPress={onPlay}
            disabled={tracks.length === 0}
            className={`w-14 h-14 rounded-full items-center justify-center bg-accent active:opacity-85 ${tracks.length === 0 ? "opacity-40" : ""}`}
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

        {/* Track list */}
        <View className="mt-3">
          {tracks.map((t, idx) => {
            const isCurrent = currentTrack?.id === t.id;
            return (
              <Pressable
                key={`${t.id}-${idx}`}
                onPress={() => playQueue(tracks, { startIdx: idx })}
                className="flex-row items-center px-5 py-2 gap-3 active:bg-bg-hover"
              >
                {t.artwork ? (
                  <Image
                    source={t.artwork}
                    className="w-11 h-11 rounded"
                    contentFit="cover"
                  />
                ) : (
                  <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
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
                    className={`text-sm font-semibold font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
                  >
                    {t.title}
                  </Text>
                  <Text
                    numberOfLines={1}
                    className="text-text-secondary text-xs mt-0.5 font-sans"
                  >
                    {t.artist} • {t.album}
                  </Text>
                </View>
                <Text className="text-text-muted text-xs font-mono">
                  {formatDuration(t.duration)}
                </Text>
                {isCurrent ? <EqualizerBars size={14} playing={isPlaying} /> : null}
                <TrackMenuButton track={t} />
              </Pressable>
            );
          })}
        </View>
      </Animated.ScrollView>

      <ActionSheet
        visible={menuOpen}
        onClose={() => setMenuOpen(false)}
        header={{
          title: playlist.name,
          subtitle: playlist.isSmart ? "Smart playlist" : "Playlist",
          image: playlist.artwork,
        }}
        actions={
          [
            {
              icon: "play-outline",
              label: "Play",
              onPress: () => {
                setMenuOpen(false);
                playQueue(tracks);
              },
              disabled: tracks.length === 0,
            },
            {
              icon: "shuffle",
              label: "Shuffle Play",
              onPress: () => {
                setMenuOpen(false);
                playQueue(tracks, { shuffle: true });
              },
              disabled: tracks.length === 0,
            },
            {
              icon: "list-outline",
              label: "Add to Queue",
              onPress: () => {
                setMenuOpen(false);
                tracks.forEach((t) => playLast(t));
              },
              disabled: tracks.length === 0,
            },
            ...(isUserPlaylist
              ? [
                  {
                    icon: "create-outline",
                    label: "Edit details",
                    onPress: () => setMenuOpen(false),
                  },
                  {
                    icon: "trash-outline",
                    label: "Delete playlist",
                    destructive: true,
                    onPress: () => setMenuOpen(false),
                  },
                ]
              : []),
            {
              icon: "share-outline",
              label: "Share",
              onPress: () => setMenuOpen(false),
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
        <RemoteServerBanner />
        <Animated.View
          pointerEvents="none"
          className="absolute top-0 left-0 right-0 bg-bg"
          style={{
            height: HEADER_HEIGHT + 64,
            opacity: headerBgOpacity,
          }}
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
            <Ionicons
              name="chevron-back"
              size={22}
              color={Colors.textPrimary}
            />
          </Pressable>
          <Animated.Text
            numberOfLines={1}
            className="flex-1 text-center text-text-primary text-[15px] font-bold px-3 font-sans"
            style={{ opacity: titleOpacity }}
          >
            {playlist.name}
          </Animated.Text>
          <View className="w-10" />
        </View>
      </SafeAreaView>
    </View>
  );
}
