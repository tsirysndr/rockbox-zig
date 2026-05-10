import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { useState } from "react";
import { Alert, FlatList, Pressable, Text, View } from "react-native";
import { SafeAreaView, useSafeAreaInsets } from "react-native-safe-area-context";

import { EqualizerBars } from "@/components/equalizer-bars";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { MiniPlayer } from "@/components/mini-player";
import { TrackMenuButton } from "@/components/track-menu-button";
import { Colors } from "@/constants/theme";
import { formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";
import { useBottomSpacing } from "@/lib/use-bottom-spacing";
import type { Track } from "@/lib/types";

type Tab = "next" | "history";

export default function QueueScreen() {
  const { queue, currentTrack, currentIdx, jumpTo, removeFromQueue, clearQueue, isPlaying } =
    usePlayer();
  const [tab, setTab] = useState<Tab>("next");
  const bottomPad = useBottomSpacing(24);
  const insets = useSafeAreaInsets();
  const floatingBottom = Math.max(insets.bottom, 8) + 4;

  // Resolve current index — if the proto stream hasn't sent one yet, fall
  // back to matching by id from currentTrack.
  const resolvedIdx =
    currentIdx ?? queue.findIndex((t) => t.id === currentTrack?.id);
  const safeIdx = resolvedIdx >= 0 ? resolvedIdx : 0;

  const playingNext = queue.slice(safeIdx + 1);
  const history = queue.slice(0, safeIdx);
  const list = tab === "next" ? playingNext : history;
  const offset = tab === "next" ? safeIdx + 1 : 0;

  return (
    <SafeAreaView className="flex-1 bg-bg">
      <RemoteServerBanner />
      <View className="flex-row items-center justify-between px-4 py-3 border-b border-divider">
        <Pressable hitSlop={8} onPress={() => router.back()}>
          <Ionicons name="chevron-down" size={26} color={Colors.textPrimary} />
        </Pressable>
        <Text className="text-text-primary text-base font-display">
          Queue
        </Text>
        <View className="flex-row items-center gap-3">
          <Text className="text-text-secondary text-xs font-sans">
            {safeIdx + 1} / {queue.length}
          </Text>
          {playingNext.length > 0 && (
            <Pressable
              hitSlop={8}
              onPress={() =>
                Alert.alert("Clear Queue", "Remove all upcoming tracks?", [
                  { text: "Cancel", style: "cancel" },
                  { text: "Clear", style: "destructive", onPress: clearQueue },
                ])
              }
            >
              <Ionicons name="trash-outline" size={20} color={Colors.textMuted} />
            </Pressable>
          )}
        </View>
      </View>

      {/* Now playing strip */}
      {currentTrack ? (
        <View className="px-4 pt-3 pb-2 border-b border-divider">
          <Text className="text-text-secondary text-[11px] font-bold uppercase tracking-widest mb-2 font-sans">
            Now playing
          </Text>
          <View className="flex-row items-center gap-3">
            {currentTrack.artwork ? (
              <Image
                source={currentTrack.artwork}
                className="w-12 h-12 rounded"
                contentFit="cover"
              />
            ) : (
              <View className="w-12 h-12 rounded bg-bg-card items-center justify-center">
                <Ionicons name="musical-note" size={18} color={Colors.textMuted} />
              </View>
            )}
            <View className="flex-1">
              <Text
                numberOfLines={1}
                className="text-accent text-sm font-display"
              >
                {currentTrack.title}
              </Text>
              <Text
                numberOfLines={1}
                className="text-text-secondary text-xs mt-0.5 font-sans"
              >
                {currentTrack.artist}
              </Text>
            </View>
            <EqualizerBars size={16} playing={isPlaying} />
          </View>
        </View>
      ) : null}

      {/* Tabs */}
      <View className="flex-row px-4 pt-3 gap-2">
        {(
          [
            { id: "next" as const, label: "Playing Next", count: playingNext.length },
            { id: "history" as const, label: "History", count: history.length },
          ]
        ).map((t) => {
          const active = t.id === tab;
          return (
            <Pressable
              key={t.id}
              onPress={() => setTab(t.id)}
              className={`h-8 px-3.5 rounded-full items-center justify-center flex-row gap-1.5 ${active ? "bg-accent" : "bg-bg-card"}`}
            >
              <Text
                className={`text-text-primary text-[13px] font-sans ${active ? "font-bold" : "font-medium"}`}
              >
                {t.label}
              </Text>
              <Text
                className={`text-[11px] font-mono ${active ? "text-white/80" : "text-text-muted"}`}
              >
                {t.count}
              </Text>
            </Pressable>
          );
        })}
      </View>

      {list.length === 0 ? (
        <View className="px-4 pt-10 items-center">
          <Ionicons
            name={tab === "next" ? "play-skip-forward-outline" : "time-outline"}
            size={36}
            color={Colors.textMuted}
          />
          <Text className="text-text-secondary text-sm mt-3 font-sans text-center">
            {tab === "next"
              ? "Nothing queued after the current track yet."
              : "No tracks have played in this session."}
          </Text>
        </View>
      ) : (
        <FlatList
          data={list}
          keyExtractor={(t, i) => `${t.id}-${offset + i}`}
          contentContainerStyle={{ paddingTop: 8, paddingBottom: bottomPad }}
          renderItem={({ item, index }) => {
            const queueIdx = offset + index;
            return (
              <QueueRow
                track={item}
                queueIdx={queueIdx}
                onPlay={() => jumpTo(queueIdx)}
                onRemove={() => removeFromQueue(queueIdx)}
              />
            );
          }}
        />
      )}
      {currentTrack ? (
        <View
          pointerEvents="box-none"
          className="absolute left-2 right-2"
          style={{ bottom: floatingBottom }}
        >
          <View
            className="bg-bg-dock rounded-xl overflow-hidden"
            style={{
              shadowColor: "#000",
              shadowOpacity: 0.5,
              shadowRadius: 16,
              shadowOffset: { width: 0, height: -4 },
            }}
          >
            <MiniPlayer />
          </View>
        </View>
      ) : null}
    </SafeAreaView>
  );
}

function QueueRow({
  track,
  queueIdx,
  onPlay,
  onRemove,
}: {
  track: Track;
  queueIdx: number;
  onPlay: () => void;
  onRemove: () => void;
}) {
  return (
    <Pressable
      onPress={onPlay}
      className="flex-row items-center gap-3 px-4 py-2.5 active:bg-bg-hover"
    >
      <Text className="w-7 text-center text-text-muted text-xs font-mono">
        {queueIdx + 1}
      </Text>
      {track.artwork ? (
        <Image
          source={track.artwork}
          className="w-11 h-11 rounded"
          contentFit="cover"
        />
      ) : (
        <View className="w-11 h-11 rounded bg-bg-card items-center justify-center">
          <Ionicons name="musical-note" size={16} color={Colors.textMuted} />
        </View>
      )}
      <View className="flex-1">
        <Text
          numberOfLines={1}
          className="text-text-primary text-sm font-medium font-sans"
        >
          {track.title}
        </Text>
        <Text
          numberOfLines={1}
          className="text-text-secondary text-xs font-sans"
        >
          {track.artist}
        </Text>
      </View>
      <Text className="text-text-muted text-xs font-mono">
        {formatDuration(track.duration)}
      </Text>
      <TrackMenuButton track={track} />
      <Pressable hitSlop={8} onPress={onRemove} className="p-1">
        <Ionicons name="close" size={18} color={Colors.textMuted} />
      </Pressable>
    </Pressable>
  );
}
