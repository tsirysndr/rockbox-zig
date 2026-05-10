import { Ionicons } from "@expo/vector-icons";
import { BlurView } from "expo-blur";
import { Image } from "expo-image";
import { router } from "expo-router";
import { useState } from "react";
import { Pressable, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

import {
  DeviceIcon,
  DevicePickerSheet,
  useCurrentDeviceLabel,
} from "@/components/device-picker";
import { RemoteServerBanner } from "@/components/remote-server-banner";
import { SeekBar } from "@/components/seek-bar";
import { Colors } from "@/constants/theme";
import { formatDuration } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

export default function PlayerScreen() {
  const {
    currentTrack,
    currentIdx,
    queue,
    position,
    isPlaying,
    shuffle,
    repeat,
    liked,
    toggle,
    next,
    prev,
    seek,
    toggleShuffle,
    cycleRepeat,
    toggleLike,
  } = usePlayer();

  const [pickerOpen, setPickerOpen] = useState(false);
  const device = useCurrentDeviceLabel();

  if (!currentTrack) {
    return (
      <SafeAreaView className="flex-1 bg-bg">
        <RemoteServerBanner />
        <View className="flex-1 items-center justify-center">
          <Text className="text-text-secondary">No track loaded</Text>
        </View>
      </SafeAreaView>
    );
  }

  const isLiked = liked.has(currentTrack.id);
  const repeatIcon =
    repeat === "one" ? "repeat" : repeat === "all" ? "repeat" : "repeat-outline";

  return (
    <View className="flex-1 bg-bg">
      {/* Full-screen album art backdrop, heavily blurred */}
      {currentTrack.artwork ? (
        <>
          <Image
            source={currentTrack.artwork}
            className="absolute inset-0"
            contentFit="cover"
            blurRadius={50}
          />
          <BlurView
            intensity={80}
            tint="dark"
            className="absolute inset-0"
          />
          {/* Bottom darken to keep controls readable */}
          <View
            className="absolute left-0 right-0 bottom-0 bg-black/55"
            style={{ height: "55%" }}
          />
        </>
      ) : null}
      <SafeAreaView className="flex-1">
        <RemoteServerBanner />
        <View className="flex-row items-center justify-between px-5 py-3">
          <Pressable hitSlop={10} onPress={() => router.back()}>
            <Ionicons name="chevron-down" size={28} color={Colors.textPrimary} />
          </Pressable>
          <View className="items-center">
            <Text className="text-text-secondary text-[11px]">
              PLAYING FROM ALBUM
            </Text>
            <Text
              numberOfLines={1}
              className="text-text-primary text-[13px] font-semibold font-sans"
            >
              {currentTrack.album}
            </Text>
          </View>
          <Pressable hitSlop={10} onPress={() => router.push("/queue")}>
            <Ionicons name="list" size={26} color={Colors.textPrimary} />
          </Pressable>
        </View>

        <View className="flex-1 items-center justify-center px-5">
          {currentTrack.artwork ? (
            <Image
              source={currentTrack.artwork}
              style={{
                width: "94%",
                aspectRatio: 1,
                borderRadius: 8,
                shadowColor: "#000",
                shadowOpacity: 0.7,
                shadowRadius: 28,
                shadowOffset: { width: 0, height: 16 },
              }}
              contentFit="cover"
            />
          ) : (
            <View
              className="bg-bg-card items-center justify-center rounded-lg"
              style={{ width: "94%", aspectRatio: 1 }}
            >
              <Ionicons
                name="musical-notes"
                size={92}
                color={Colors.textMuted}
              />
            </View>
          )}
        </View>

        <View className="px-6 pb-4">
          <View className="flex-row items-end justify-between mb-1.5">
            <View className="flex-1 pr-3">
              <Text
                numberOfLines={1}
                className="text-text-primary text-[22px] font-display-extra"
              >
                {currentTrack.title}
              </Text>
              <Text
                numberOfLines={1}
                className="text-text-secondary text-[15px] mt-0.5 font-sans"
              >
                {currentTrack.artist}
              </Text>
            </View>
            <Pressable hitSlop={10} onPress={() => toggleLike(currentTrack.id)}>
              <Ionicons
                name={isLiked ? "heart" : "heart-outline"}
                size={28}
                color={isLiked ? Colors.accent : Colors.textPrimary}
              />
            </Pressable>
          </View>

          <View className="mt-3.5">
            <SeekBar
              value={position}
              max={currentTrack.duration}
              onSeek={(v) => seek(v)}
              fill="#FFFFFF"
            />
            <View className="flex-row justify-between mt-1">
              <Text className="text-text-muted text-[11px] font-mono">
                {formatDuration(position)}
              </Text>
              <Text className="text-text-muted text-[11px] font-mono">
                {formatDuration(currentTrack.duration)}
              </Text>
            </View>
          </View>

          <View className="flex-row items-center justify-between mt-4 px-1">
            <Pressable hitSlop={10} onPress={toggleShuffle}>
              <Ionicons
                name="shuffle"
                size={22}
                color={shuffle ? Colors.accent : Colors.textSecondary}
              />
            </Pressable>
            <Pressable hitSlop={10} onPress={prev}>
              <Ionicons
                name="play-skip-back"
                size={32}
                color={Colors.textPrimary}
              />
            </Pressable>
            <Pressable
              onPress={toggle}
              className="w-[68px] h-[68px] rounded-full bg-accent items-center justify-center"
            >
              <Ionicons
                name={isPlaying ? "pause" : "play"}
                size={32}
                color="#FFFFFF"
                style={{ marginLeft: isPlaying ? 0 : 3 }}
              />
            </Pressable>
            <Pressable hitSlop={10} onPress={next}>
              <Ionicons
                name="play-skip-forward"
                size={32}
                color={Colors.textPrimary}
              />
            </Pressable>
            <Pressable hitSlop={10} onPress={cycleRepeat}>
              <Ionicons
                name={repeatIcon as any}
                size={22}
                color={repeat === "off" ? Colors.textSecondary : Colors.accent}
              />
              {repeat === "one" ? (
                <Text className="absolute top-1.5 -right-1.5 text-accent text-[9px] font-extrabold font-mono">
                  1
                </Text>
              ) : null}
            </Pressable>
          </View>

          <View className="flex-row items-center justify-between mt-4">
            <Text className="text-text-muted text-[11px] font-sans">
              {currentIdx + 1} / {queue.length}
            </Text>
            <Pressable
              hitSlop={8}
              onPress={() => setPickerOpen(true)}
              className="flex-row items-center gap-1.5"
            >
              <DeviceIcon
                spec={device.icon}
                size={16}
                color={Colors.textMuted}
              />
              <Text className="text-text-muted text-[11px] font-sans">
                {device.name}
              </Text>
            </Pressable>
          </View>
        </View>
      </SafeAreaView>
      <DevicePickerSheet
        visible={pickerOpen}
        onClose={() => setPickerOpen(false)}
      />
    </View>
  );
}
