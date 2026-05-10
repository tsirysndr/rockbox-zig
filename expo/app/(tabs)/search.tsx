import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { useState } from "react";
import { Pressable, ScrollView, Text, TextInput, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import { NotConnectedState } from "@/components/empty-state";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { EqualizerBars } from "@/components/equalizer-bars";
import { TrackMenuButton } from "@/components/track-menu-button";
import { useIsConnected } from "@/lib/connection";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import { Colors } from "@/constants/theme";
import { useLibraryGenres, useLibrarySearch } from "@/lib/library-source";
import { formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

export default function SearchScreen() {
  const [query, setQuery] = useState("");
  const { playTrack, currentTrack, isPlaying } = usePlayer();
  const isConnected = useIsConnected();
  const bottomPad = useBottomSpacing(24);
  const { data: results } = useLibrarySearch(query);
  const {
    data: genres,
    isLoading: genresLoading,
    error: genresError,
  } = useLibraryGenres();
  const tracks = results.tracks;

  return (
    <SafeAreaView className="flex-1 bg-bg" edges={["top"]}>
      <RemoteServerBanner />
      <View className="px-4 pt-2 pb-4">
        <Text className="text-text-primary text-[26px] mb-4 font-display-extra">
          Search
        </Text>
        <View className="flex-row items-center bg-bg-card rounded-md px-3 h-11 gap-2">
          <Ionicons name="search" size={18} color={Colors.textMuted} />
          <TextInput
            value={query}
            onChangeText={setQuery}
            placeholder="Songs, artists, albums"
            placeholderTextColor={Colors.textMuted}
            className="flex-1 text-text-primary text-[15px] font-sans"
            autoCorrect={false}
            returnKeyType="search"
          />
          {query.length > 0 ? (
            <Pressable hitSlop={6} onPress={() => setQuery("")}>
              <Ionicons
                name="close-circle"
                size={18}
                color={Colors.textMuted}
              />
            </Pressable>
          ) : null}
        </View>
      </View>

      {!isConnected ? (
        <NotConnectedState message="Connect to a server to search your library." />
      ) : query.trim().length > 0 ? (
        <ScrollView contentContainerStyle={{ paddingBottom: bottomPad }}>
          {tracks.length === 0 &&
          results.albums.length === 0 &&
          results.artists.length === 0 ? (
            <Text className="text-text-secondary px-4 mt-6 font-sans">
              No results for &ldquo;{query}&rdquo;
            </Text>
          ) : (
            <>
              {results.artists.length > 0 ? (
                <View className="px-4 pt-2 pb-1">
                  <Text className="text-text-primary text-base font-display mb-1.5">
                    Artists
                  </Text>
                </View>
              ) : null}
              {results.artists.map((a) => (
                <Pressable
                  key={`artist-${a.id}`}
                  onPress={() => router.push(`/artist/${a.id}`)}
                  className="flex-row items-center px-4 py-2 gap-3 active:bg-bg-hover"
                >
                  {a.image ? (
                    <Image
                      source={a.image}
                      className="w-11 h-11 rounded-full"
                      contentFit="cover"
                    />
                  ) : (
                    <View className="w-11 h-11 bg-bg-card rounded-full items-center justify-center">
                      <Ionicons
                        name="person"
                        size={18}
                        color={Colors.textMuted}
                      />
                    </View>
                  )}
                  <Text
                    numberOfLines={1}
                    className="text-text-primary text-sm font-semibold font-sans flex-1"
                  >
                    {a.name}
                  </Text>
                </Pressable>
              ))}

              {results.albums.length > 0 ? (
                <View className="px-4 pt-3 pb-1">
                  <Text className="text-text-primary text-base font-display mb-1.5">
                    Albums
                  </Text>
                </View>
              ) : null}
              {results.albums.map((al) => (
                <Pressable
                  key={`album-${al.id}`}
                  onPress={() => router.push(`/album/${al.id}`)}
                  className="flex-row items-center px-4 py-2 gap-3 active:bg-bg-hover"
                >
                  {al.artwork ? (
                    <Image
                      source={al.artwork}
                      className="w-11 h-11 rounded"
                      contentFit="cover"
                    />
                  ) : (
                    <View className="w-11 h-11 bg-bg-card rounded items-center justify-center">
                      <Ionicons name="disc" size={18} color={Colors.textMuted} />
                    </View>
                  )}
                  <View className="flex-1">
                    <Text
                      numberOfLines={1}
                      className="text-text-primary text-sm font-semibold font-sans"
                    >
                      {al.title}
                    </Text>
                    <Text
                      numberOfLines={1}
                      className="text-text-secondary text-xs font-sans"
                    >
                      {al.artist}
                    </Text>
                  </View>
                </Pressable>
              ))}

              {tracks.length > 0 ? (
                <View className="px-4 pt-3 pb-1">
                  <Text className="text-text-primary text-base font-display mb-1.5">
                    Songs
                  </Text>
                </View>
              ) : null}
              {tracks.map((track) => {
                const isCurrent =
                  currentTrack?.id === track.id && !!track.id;
                return (
                  <Pressable
                    key={`track-${track.id}`}
                    onPress={() => playTrack(track)}
                    className="flex-row items-center px-4 py-2.5 gap-3 active:bg-bg-hover"
                  >
                    {track.artwork ? (
                      <Image
                        source={track.artwork}
                        className="w-11 h-11 rounded-md"
                        contentFit="cover"
                      />
                    ) : (
                      <View className="w-11 h-11 bg-bg-card rounded-md items-center justify-center">
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
                        {track.title}
                      </Text>
                      <Text
                        numberOfLines={1}
                        className="text-text-secondary text-xs font-sans"
                      >
                        {track.artist} • {track.album}
                      </Text>
                    </View>
                    <Text className="text-text-muted text-xs font-mono">
                      {formatDuration(track.duration)}
                    </Text>
                    {isCurrent ? (
                      <EqualizerBars size={14} playing={isPlaying} />
                    ) : null}
                    <TrackMenuButton track={track} />
                  </Pressable>
                );
              })}
            </>
          )}
        </ScrollView>
      ) : (
        <ScrollView
          contentContainerStyle={{ paddingHorizontal: 16, paddingBottom: bottomPad }}
          showsVerticalScrollIndicator={false}
        >
          <Text className="text-text-primary text-lg mb-3 font-display">
            Browse all
          </Text>
          {genresError ? (
            <Text className="text-danger text-sm font-sans">
              Genres failed: {String((genresError as Error)?.message ?? genresError)}
            </Text>
          ) : genres.length === 0 ? (
            <Text className="text-text-secondary text-sm font-sans">
              {genresLoading
                ? "Loading genres…"
                : "No genres yet — wait for the daemon to finish scanning."}
            </Text>
          ) : (
            <View className="flex-row flex-wrap gap-2">
              {genres.map((g) => (
                <Pressable
                  key={g.id}
                  onPress={() => router.push(`/genre/${encodeURIComponent(g.id)}`)}
                  style={{ backgroundColor: g.color }}
                  className="w-[48.5%] h-[100px] rounded-md p-3 overflow-hidden active:opacity-80"
                >
                  <Text className="text-white text-lg font-display">
                    {g.name}
                  </Text>
                </Pressable>
              ))}
            </View>
          )}
        </ScrollView>
      )}
    </SafeAreaView>
  );
}
