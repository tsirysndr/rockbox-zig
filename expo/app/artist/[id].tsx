import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { router, useLocalSearchParams } from "expo-router";
import { useMemo, useState } from "react";
import {
  Animated,
  Dimensions,
  FlatList,
  Pressable,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { ActionSheet, type ActionItem } from "@/components/action-sheet";
import { EqualizerBars } from "@/components/equalizer-bars";
import { ArtistPlaceholder } from "@/components/artist-placeholder";
import { artistGradientColors, heroGradientColors, gradientIconColor } from "@/components/playlist-cover";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { TrackMenuButton } from "@/components/track-menu-button";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import { useArtistDetail } from "@/lib/library-source";
import { formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

const { width } = Dimensions.get("window");
const HERO_HEIGHT = Math.min(width, 360);
const HEADER_HEIGHT = 56;
const TOP_TRACK_LIMIT = 5;

export default function ArtistScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const detail = useArtistDetail(id ?? "");
  const artist = detail.artist;
  const tracks = detail.tracks;
  const albums = detail.albums;
  const { playQueue, currentTrack, isPlaying, playLast } = usePlayer();
  const topTracks = tracks.slice(0, TOP_TRACK_LIMIT);
  const [menuOpen, setMenuOpen] = useState(false);
  const bottomPad = useBottomSpacing(24);

  const scrollY = useMemo(() => new Animated.Value(0), []);
  const headerBgOpacity = scrollY.interpolate({
    inputRange: [HERO_HEIGHT - 120, HERO_HEIGHT - 40],
    outputRange: [0, 1],
    extrapolate: "clamp",
  });
  const titleOpacity = scrollY.interpolate({
    inputRange: [HERO_HEIGHT - 80, HERO_HEIGHT - 30],
    outputRange: [0, 1],
    extrapolate: "clamp",
  });
  const heroTranslate = scrollY.interpolate({
    inputRange: [-200, 0, HERO_HEIGHT],
    outputRange: [-100, 0, HERO_HEIGHT * 0.5],
    extrapolate: "clamp",
  });
  const heroScale = scrollY.interpolate({
    inputRange: [-200, 0],
    outputRange: [1.4, 1],
    extrapolateRight: "clamp",
  });

  if (!artist) {
    return (
      <SafeAreaView className="flex-1 bg-bg">
        <View className="flex-1 items-center justify-center">
          <Text className="text-text-secondary">Artist not found</Text>
        </View>
      </SafeAreaView>
    );
  }

  const onPlay = () => playQueue(tracks);
  const onShuffle = () => playQueue(tracks, { shuffle: true });

  return (
    <View className="flex-1 bg-bg">
      {/* Parallax hero */}
      <Animated.View
        className="absolute top-0 left-0 right-0"
        style={{
          height: HERO_HEIGHT,
          transform: [{ translateY: heroTranslate }, { scale: heroScale }],
        }}
      >
        {artist.image ? (
          <Image
            source={artist.image}
            className="w-full h-full"
            contentFit="cover"
          />
        ) : (
          <LinearGradient
            colors={artistGradientColors(artist.name)}
            start={{ x: 0, y: 0 }}
            end={{ x: 1, y: 1 }}
            style={{ flex: 1, alignItems: "center", justifyContent: "center", paddingBottom: 90 }}
          >
            <ArtistPlaceholder size={88} color={gradientIconColor(artistGradientColors(artist.name))} />
          </LinearGradient>
        )}
        <LinearGradient
          colors={[
            "rgba(0,0,0,0)",
            "rgba(0,0,0,0.25)",
            "rgba(0,0,0,0.7)",
            "rgba(0,0,0,1)",
          ]}
          locations={[0, 0.4, 0.75, 1]}
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
          style={{ height: HERO_HEIGHT - 80 }}
        >
          <View className="px-5 pb-4">
            <Text
              className="text-text-primary text-3xl font-display-extra"
              style={{
                textShadowColor: "rgba(0,0,0,0.6)",
                textShadowRadius: 8,
              }}
            >
              {artist.name}
            </Text>
          </View>
        </View>

        <View className="bg-bg pt-4">
          <View className="flex-row items-center px-5 gap-4">
            <Pressable hitSlop={6} onPress={onShuffle}>
              <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
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

          {topTracks.length > 0 ? (
            <View className="mt-7">
              <Text className="text-text-primary text-lg font-bold px-5 mb-2 font-sans">
                Popular
              </Text>
              {topTracks.map((t, idx) => {
                const isCurrent = currentTrack?.id === t.id;
                return (
                  <Pressable
                    key={t.id}
                    onPress={() => playQueue(tracks, { startIdx: idx })}
                    className="flex-row items-center px-5 py-2 gap-3 active:bg-bg-hover"
                  >
                    <View className="w-[18px] items-center justify-center">
                      {isCurrent ? (
                        <EqualizerBars size={14} playing={isPlaying} />
                      ) : (
                        <Text className="text-text-muted text-sm font-mono">
                          {idx + 1}
                        </Text>
                      )}
                    </View>
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
                        {t.album}
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
          ) : null}

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
                    {item.artwork ? (
                      <Image
                        source={item.artwork}
                        className="w-[150px] h-[150px] rounded-md"
                        contentFit="cover"
                      />
                    ) : (
                      <View className="w-[150px] h-[150px] rounded-md overflow-hidden">
                        <LinearGradient
                          colors={heroGradientColors(item.title)}
                          start={{ x: 0, y: 0 }}
                          end={{ x: 1, y: 1 }}
                          style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
                        >
                          <View style={{ opacity: 0.4 }}>
                            <Ionicons
                              name="disc"
                              size={Math.round(150 * 0.42)}
                              color={gradientIconColor(heroGradientColors(item.title))}
                            />
                          </View>
                        </LinearGradient>
                      </View>
                    )}
                    <Text
                      numberOfLines={1}
                      className="text-text-primary text-sm font-semibold mt-2 font-sans"
                    >
                      {item.title}
                    </Text>
                    <Text className="text-text-secondary text-xs mt-0.5 font-sans">
                      Album{item.year ? ` • ${item.year}` : ""}
                    </Text>
                  </Pressable>
                )}
              />
            </View>
          ) : null}
        </View>
      </Animated.ScrollView>

      <ActionSheet
        visible={menuOpen}
        onClose={() => setMenuOpen(false)}
        header={{
          title: artist.name,
          subtitle: "Artist",
          image: artist.image,
          rounded: "full",
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
              label: "Add Top Songs to Queue",
              onPress: () => {
                setMenuOpen(false);
                topTracks.forEach((t) => playLast(t));
              },
              disabled: topTracks.length === 0,
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
            {artist.name}
          </Animated.Text>
          <View className="w-10" />
        </View>
      </SafeAreaView>
    </View>
  );
}
