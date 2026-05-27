import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { router, useLocalSearchParams } from "expo-router";
import { useMemo } from "react";
import {
  Animated,
  Dimensions,
  Pressable,
  ScrollView,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { ArtistPlaceholder } from "@/components/artist-placeholder";
import { Colors } from "@/constants/theme";
import { coverArtUrl } from "@/lib/navidrome-client";
import { useNdArtistDetail } from "@/lib/navidrome-source";
import { useNdActiveServer } from "@/lib/navidrome-store";
import { usePlayer } from "@/lib/player-context";
import type { Track } from "@/lib/types";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";

const { width } = Dimensions.get("window");
const ART_SIZE = Math.min(width * 0.55, 240);

export default function NdArtistScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const nd = useNdActiveServer();
  const { data: detail, isLoading } = useNdArtistDetail(id ?? "");
  const { playQueue } = usePlayer();
  const bottomPad = useBottomSpacing(24);

  const scrollY = useMemo(() => new Animated.Value(0), []);
  const artOpacity = scrollY.interpolate({
    inputRange: [0, 200, 320],
    outputRange: [1, 1, 0],
    extrapolate: "clamp",
  });

  const artist = detail?.artist;
  const albums = detail?.albums ?? [];

  const artSrc = artist?.coverArt && nd
    ? coverArtUrl(nd.baseUrl, nd.user, nd.password, artist.coverArt, 500)
    : null;

  // Shuffle all album tracks by building fake Track objects for the stream URLs.
  // We don't pre-load all tracks here; we'll just play the first album.
  const onShuffle = () => {
    if (!albums[0]) return;
    router.push(`/nd-album/${albums[0].id}` as any);
  };

  const colCount = 2;
  const itemWidth = (width - 48 - 12) / colCount;

  return (
    <View className="flex-1 bg-bg">
      {/* Back button */}
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
          style={{ opacity: artOpacity }}
          className="items-center pt-16 pb-6"
        >
          {artSrc ? (
            <View style={{ width: ART_SIZE, height: ART_SIZE, borderRadius: ART_SIZE / 2, overflow: "hidden" }}>
              <Image
                source={{ uri: artSrc }}
                className="w-full h-full"
                contentFit="cover"
              />
            </View>
          ) : (
            <View
              style={{ width: ART_SIZE, height: ART_SIZE, borderRadius: ART_SIZE / 2 }}
              className="bg-bg-card items-center justify-center"
            >
              <ArtistPlaceholder size={64} />
            </View>
          )}
        </Animated.View>

        {/* Name + actions */}
        <View className="px-5 pb-5">
          <Text
            className="text-text-primary text-[26px] font-display-extra text-center"
            numberOfLines={2}
          >
            {artist?.name ?? "Loading…"}
          </Text>
          <Text className="text-text-secondary text-[14px] text-center mt-1 font-sans">
            {albums.length} {albums.length === 1 ? "album" : "albums"}
          </Text>
          <View className="flex-row items-center justify-center gap-4 mt-5">
            <Pressable
              hitSlop={6}
              onPress={onShuffle}
              disabled={albums.length === 0}
            >
              <Ionicons name="shuffle" size={26} color={Colors.textPrimary} />
            </Pressable>
          </View>
        </View>

        {/* Albums grid */}
        <View className="px-4 pb-4">
          <Text className="text-text-secondary text-xs font-bold uppercase tracking-widest mb-3 font-sans">
            Albums
          </Text>
          {isLoading ? (
            <View className="flex-row flex-wrap gap-3">
              {Array.from({ length: 6 }, (_, i) => (
                <View key={i} style={{ width: itemWidth }}>
                  <View
                    style={{ width: itemWidth, height: itemWidth, borderRadius: 8 }}
                    className="bg-bg-card"
                  />
                  <View className="w-3/4 h-3 rounded bg-bg-card mt-1.5" />
                  <View className="w-1/2 h-2.5 rounded bg-bg-card mt-1" />
                </View>
              ))}
            </View>
          ) : (
            <View className="flex-row flex-wrap gap-3">
              {albums.map((album) => {
                const artSrcAlbum = album.coverArt && nd
                  ? coverArtUrl(nd.baseUrl, nd.user, nd.password, album.coverArt, 300)
                  : null;
                return (
                  <Pressable
                    key={album.id}
                    onPress={() => router.push(`/nd-album/${album.id}` as any)}
                    style={{ width: itemWidth }}
                    className="active:opacity-80"
                  >
                    {artSrcAlbum ? (
                      <View style={{ width: itemWidth, height: itemWidth, borderRadius: 8, overflow: "hidden" }}>
                        <Image
                          source={{ uri: artSrcAlbum }}
                          className="w-full h-full"
                          contentFit="cover"
                        />
                      </View>
                    ) : (
                      <View
                        style={{ width: itemWidth, height: itemWidth, borderRadius: 8, overflow: "hidden" }}
                      >
                        <LinearGradient
                          colors={["#1A1B2E", "#2D1B69"]}
                          style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
                        >
                          <Ionicons name="disc" size={40} color="rgba(255,255,255,0.3)" />
                        </LinearGradient>
                      </View>
                    )}
                    <Text
                      numberOfLines={1}
                      className="text-text-primary text-[13px] font-semibold mt-1.5 font-sans"
                    >
                      {album.name}
                    </Text>
                    <Text numberOfLines={1} className="text-text-secondary text-xs font-sans">
                      {album.year ?? ""}
                    </Text>
                  </Pressable>
                );
              })}
            </View>
          )}
        </View>

        <View style={{ height: bottomPad }} />
      </Animated.ScrollView>
    </View>
  );
}
