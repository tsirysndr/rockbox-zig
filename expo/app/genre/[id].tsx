import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { router, useLocalSearchParams } from "expo-router";
import { useMemo } from "react";
import {
  Animated,
  FlatList,
  Pressable,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { TrackMenuButton } from "@/components/track-menu-button";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import {
  formatDuration,
  getGenreAlbums,
  getGenreArtists,
  getGenreById,
  getGenreTracks,
} from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

const HEADER_HEIGHT = 56;
const HERO_HEIGHT = 220;
const TOP_TRACK_LIMIT = 5;

export default function GenreScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const genre = id ? getGenreById(id) : undefined;
  const tracks = useMemo(() => (id ? getGenreTracks(id) : []), [id]);
  const albums = useMemo(() => (id ? getGenreAlbums(id) : []), [id]);
  const artists = useMemo(() => (id ? getGenreArtists(id) : []), [id]);
  const { playQueue, currentTrack, isPlaying } = usePlayer();
  const topTracks = tracks.slice(0, TOP_TRACK_LIMIT);
  const bottomPad = useBottomSpacing(24);

  const scrollY = useMemo(() => new Animated.Value(0), []);
  const headerBgOpacity = scrollY.interpolate({
    inputRange: [HERO_HEIGHT - 100, HERO_HEIGHT - 20],
    outputRange: [0, 1],
    extrapolate: "clamp",
  });
  const titleOpacity = scrollY.interpolate({
    inputRange: [HERO_HEIGHT - 60, HERO_HEIGHT - 10],
    outputRange: [0, 1],
    extrapolate: "clamp",
  });
  const heroTranslate = scrollY.interpolate({
    inputRange: [-200, 0, HERO_HEIGHT],
    outputRange: [-100, 0, HERO_HEIGHT * 0.5],
    extrapolate: "clamp",
  });

  if (!genre) {
    return (
      <SafeAreaView className="flex-1 bg-bg">
        <View className="flex-1 items-center justify-center">
          <Text className="text-text-secondary">Genre not found</Text>
        </View>
      </SafeAreaView>
    );
  }

  const onPlay = () => playQueue(tracks);
  const onShuffle = () => playQueue(tracks, { shuffle: true });

  return (
    <View className="flex-1 bg-bg">
      {/* Parallax color hero */}
      <Animated.View
        className="absolute top-0 left-0 right-0 overflow-hidden"
        style={{
          height: HERO_HEIGHT,
          backgroundColor: genre.color,
          transform: [{ translateY: heroTranslate }],
        }}
      >
        <Text
          className="absolute right-4 bottom-4 text-white/30 text-[80px] font-display-extra"
          style={{ transform: [{ rotate: "-12deg" }] }}
          numberOfLines={1}
        >
          {genre.name}
        </Text>
        <LinearGradient
          colors={[
            "rgba(0,0,0,0)",
            "rgba(0,0,0,0.35)",
            "rgba(0,0,0,1)",
          ]}
          locations={[0, 0.55, 1]}
          className="absolute inset-0"
        />
      </Animated.View>

      <Animated.ScrollView
        showsVerticalScrollIndicator={false}
        scrollEventThrottle={16}
        onScroll={Animated.event(
          [{ nativeEvent: { contentOffset: { y: scrollY } } }],
          { useNativeDriver: true },
        )}
        contentContainerStyle={{ paddingBottom: bottomPad }}
      >
        <View
          className="justify-end"
          style={{ height: HERO_HEIGHT - 60 }}
        >
          <View className="px-5 pb-3">
            <Text className="text-white/80 text-xs font-bold tracking-widest uppercase font-sans">
              Genre
            </Text>
            <Text
              className="text-text-primary text-[34px] font-extrabold mt-1 font-sans"
              style={{
                textShadowColor: "rgba(0,0,0,0.6)",
                textShadowRadius: 8,
              }}
            >
              {genre.name}
            </Text>
          </View>
        </View>

        <View className="bg-bg pt-4">
          <View className="px-5 flex-row items-center gap-4">
            <Text className="text-text-secondary text-[13px] flex-1 font-sans">
              {tracks.length} tracks · {albums.length} albums ·{" "}
              {artists.length} artists
            </Text>
            <Pressable
              hitSlop={6}
              onPress={onShuffle}
              disabled={tracks.length === 0}
            >
              <Ionicons
                name="shuffle"
                size={26}
                color={
                  tracks.length === 0 ? Colors.textMuted : Colors.textPrimary
                }
              />
            </Pressable>
            <Pressable
              onPress={onPlay}
              disabled={tracks.length === 0}
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

          {topTracks.length > 0 ? (
            <View className="mt-7">
              <Text className="text-text-primary text-lg font-bold px-5 mb-2 font-sans">
                Popular tracks
              </Text>
              {topTracks.map((t, idx) => {
                const isCurrent = currentTrack?.id === t.id;
                return (
                  <Pressable
                    key={t.id}
                    onPress={() => playQueue(tracks, { startIdx: idx })}
                    className="flex-row items-center px-5 py-2 gap-3 active:bg-bg-hover"
                  >
                    <Text className="w-[18px] text-center text-text-muted text-sm font-mono">
                      {idx + 1}
                    </Text>
                    {t.artwork ? (
                      <Image
                        source={t.artwork}
                        className="w-11 h-11 rounded"
                        contentFit="cover"
                      />
                    ) : null}
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
                        {t.artist}
                      </Text>
                    </View>
                    <Text className="text-text-muted text-xs font-mono">
                      {formatDuration(t.duration)}
                    </Text>
                    {isCurrent && isPlaying ? (
                      <Ionicons
                        name="musical-notes"
                        size={14}
                        color={Colors.accent}
                      />
                    ) : null}
                    <TrackMenuButton track={t} />
                  </Pressable>
                );
              })}
            </View>
          ) : (
            <View className="px-5 mt-8">
              <Text className="text-text-secondary text-sm font-sans">
                No content available in {genre.name} yet.
              </Text>
            </View>
          )}

          {albums.length > 0 ? (
            <View className="mt-8">
              <Text className="text-text-primary text-lg font-bold px-5 mb-3 font-sans">
                Albums
              </Text>
              <FlatList
                horizontal
                data={albums}
                keyExtractor={(a) => a.id}
                showsHorizontalScrollIndicator={false}
                contentContainerStyle={{ paddingHorizontal: 20, gap: 14 }}
                renderItem={({ item }) => (
                  <Pressable
                    onPress={() => router.push(`/album/${item.id}`)}
                    className="w-[150px] active:opacity-85"
                  >
                    <Image
                      source={item.artwork}
                      className="w-[150px] h-[150px] rounded-md"
                      contentFit="cover"
                    />
                    <Text
                      numberOfLines={1}
                      className="text-text-primary text-sm font-semibold mt-2 font-sans"
                    >
                      {item.title}
                    </Text>
                    <Text
                      numberOfLines={1}
                      className="text-text-secondary text-xs mt-0.5 font-sans"
                    >
                      {item.artist}
                    </Text>
                  </Pressable>
                )}
              />
            </View>
          ) : null}

          {artists.length > 0 ? (
            <View className="mt-8">
              <Text className="text-text-primary text-lg font-bold px-5 mb-3 font-sans">
                Artists
              </Text>
              <FlatList
                horizontal
                data={artists}
                keyExtractor={(a) => a.id}
                showsHorizontalScrollIndicator={false}
                contentContainerStyle={{ paddingHorizontal: 20, gap: 14 }}
                renderItem={({ item }) => (
                  <Pressable
                    onPress={() => router.push(`/artist/${item.id}`)}
                    className="w-[120px] items-center active:opacity-85"
                  >
                    <Image
                      source={item.image}
                      className="w-[120px] h-[120px] rounded-full"
                      contentFit="cover"
                    />
                    <Text
                      numberOfLines={1}
                      className="text-text-primary text-sm font-semibold mt-2 font-sans"
                    >
                      {item.name}
                    </Text>
                    <Text
                      numberOfLines={1}
                      className="text-text-secondary text-xs mt-0.5 font-sans"
                    >
                      Artist
                    </Text>
                  </Pressable>
                )}
              />
            </View>
          ) : null}
        </View>
      </Animated.ScrollView>

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
            {genre.name}
          </Animated.Text>
          <View className="w-10" />
        </View>
      </SafeAreaView>
    </View>
  );
}
