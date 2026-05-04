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
import { TrackMenuButton } from "@/components/track-menu-button";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import { useAlbumDetail } from "@/lib/library-source";
import { ARTISTS, formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

const { width } = Dimensions.get("window");
const ART_SIZE = Math.min(width * 0.62, 280);
const HEADER_HEIGHT = 56;

export default function AlbumScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const detail = useAlbumDetail(id ?? "");
  const album = detail.album;
  const tracks = detail.tracks;
  const { playQueue, currentTrack, isPlaying, playLast } = usePlayer();
  const [menuOpen, setMenuOpen] = useState(false);

  const totalDuration = tracks.reduce((s, t) => s + t.duration, 0);
  const totalMinutes = Math.round(totalDuration / 60);
  const bottomPad = useBottomSpacing(24);

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

  if (!album) {
    return (
      <SafeAreaView className="flex-1 bg-bg">
        <View className="flex-1 items-center justify-center">
          <Text className="text-text-secondary">Album not found</Text>
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
        {/* Hero band: blurred album art behind, sharp art front-and-center */}
        <View
          className="items-center overflow-hidden pb-4"
          style={{ paddingTop: HEADER_HEIGHT + 16 }}
        >
          <Image
            source={album.artwork}
            className="absolute inset-0"
            contentFit="cover"
            blurRadius={40}
          />
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
            <Image
              source={album.artwork}
              className="rounded-lg"
              style={{ width: ART_SIZE, height: ART_SIZE }}
              contentFit="cover"
            />
          </Animated.View>
        </View>

        {/* Title block */}
        <View className="px-5 mt-3.5">
          <Text className="text-text-primary text-[26px] font-display-extra">
            {album.title}
          </Text>
          <Pressable
            hitSlop={6}
            onPress={() => {
              const ar = ARTISTS.find((a) => a.name === album.artist);
              if (ar) router.push(`/artist/${ar.id}`);
            }}
            className="mt-1.5"
          >
            <Text className="text-text-primary text-sm font-semibold font-sans">
              {album.artist}
            </Text>
          </Pressable>
          <Text className="text-text-secondary text-xs mt-1 font-sans">
            Album{album.year ? ` • ${album.year}` : ""} • {tracks.length}{" "}
            tracks • {totalMinutes} min
          </Text>
        </View>

        {/* Action row */}
        <View className="flex-row items-center px-5 mt-5 gap-4">
          <Pressable
            hitSlop={6}
            onPress={onShuffle}
            className="w-11 h-11 rounded-full items-center justify-center"
          >
            <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
          </Pressable>
          <Pressable hitSlop={6}>
            <Ionicons
              name="heart-outline"
              size={26}
              color={Colors.textPrimary}
            />
          </Pressable>
          <Pressable hitSlop={6} onPress={() => setMenuOpen(true)}>
            <Ionicons
              name="ellipsis-horizontal"
              size={26}
              color={Colors.textPrimary}
            />
          </Pressable>
          <View className="flex-1" />
          <Pressable
            onPress={onPlay}
            className="w-14 h-14 rounded-full items-center justify-center bg-accent active:opacity-85"
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
        <View className="mt-5">
          {tracks.map((t, idx) => {
            const isCurrent = currentTrack?.id === t.id;
            return (
              <Pressable
                key={t.id}
                onPress={() => playQueue(tracks, { startIdx: idx })}
                className="flex-row items-center px-5 py-2.5 gap-3.5 active:bg-bg-hover"
              >
                <View className="w-[22px] items-center">
                  {isCurrent ? (
                    <EqualizerBars size={14} playing={isPlaying} />
                  ) : (
                    <Text className="text-text-muted text-[13px] font-mono">
                      {idx + 1}
                    </Text>
                  )}
                </View>
                <View className="flex-1">
                  <Text
                    numberOfLines={1}
                    className={`text-[15px] font-medium font-sans ${isCurrent ? "text-accent" : "text-text-primary"}`}
                  >
                    {t.title}
                  </Text>
                  <Text
                    numberOfLines={1}
                    className="text-text-secondary text-xs mt-0.5 font-sans"
                  >
                    {t.artist}
                  </Text>
                </View>
                <Text className="text-text-muted text-xs font-mono">
                  {formatDuration(t.duration)}
                </Text>
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
          title: album.title,
          subtitle: album.artist,
          image: album.artwork,
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
            },
            {
              icon: "shuffle",
              label: "Shuffle Play",
              onPress: () => {
                setMenuOpen(false);
                playQueue(tracks, { shuffle: true });
              },
            },
            {
              icon: "list-outline",
              label: "Add to Queue",
              onPress: () => {
                setMenuOpen(false);
                tracks.forEach((t) => playLast(t));
              },
            },
            {
              icon: "add-circle-outline",
              label: "Add to Playlist",
              onPress: () => setMenuOpen(false),
            },
            {
              icon: "person-outline",
              label: "Go to Artist",
              onPress: () => {
                const ar = ARTISTS.find((a) => a.name === album.artist);
                setMenuOpen(false);
                if (ar) router.push(`/artist/${ar.id}`);
              },
            },
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
            {album.title}
          </Animated.Text>
          <View className="w-10" />
        </View>
      </SafeAreaView>
    </View>
  );
}
