import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { useState } from "react";
import { Alert, Modal, Pressable, ScrollView, Text, View } from "react-native";

import { PlaylistCover } from "@/components/playlist-cover";
import { Colors } from "@/constants/theme";
import { useLibraryPlaylists } from "@/lib/library-source";
import { usePlayer } from "@/lib/player-context";
import { RockboxClient } from "@/lib/rockbox-client";

export function TrackContextMenu() {
  const {
    contextTrack,
    closeContextMenu,
    playNext,
    playLast,
    toggleLike,
    liked,
  } = usePlayer();

  const open = contextTrack !== null;
  const track = contextTrack;
  const [pickerOpen, setPickerOpen] = useState(false);
  const { data: playlists } = useLibraryPlaylists();

  if (!track) {
    return (
      <Modal
        visible={open}
        transparent
        animationType="fade"
        onRequestClose={closeContextMenu}
      />
    );
  }

  const isLiked = liked.has(track.id);
  const artistId = track.artistId;
  const albumId = track.albumId;

  const items: {
    icon: React.ComponentProps<typeof Ionicons>["name"];
    label: string;
    onPress?: () => void;
    disabled?: boolean;
    destructive?: boolean;
  }[] = [
    {
      icon: isLiked ? "heart" : "heart-outline",
      label: isLiked ? "Remove from Liked" : "Add to Liked",
      onPress: () => {
        toggleLike(track.id);
        closeContextMenu();
      },
    },
    {
      icon: "play-skip-forward-outline",
      label: "Play Next",
      onPress: () => {
        playNext(track);
        closeContextMenu();
      },
    },
    {
      icon: "list-outline",
      label: "Play Last",
      onPress: () => {
        playLast(track);
        closeContextMenu();
      },
    },
    {
      icon: "add-circle-outline",
      label: "Add to Playlist",
      onPress: () => {
        setPickerOpen(true);
      },
    },
    {
      icon: "person-outline",
      label: "Go to Artist",
      disabled: !artistId,
      onPress: () => {
        if (!artistId) return;
        closeContextMenu();
        router.push(`/artist/${artistId}`);
      },
    },
    {
      icon: "disc-outline",
      label: "Go to Album",
      disabled: !albumId,
      onPress: () => {
        if (!albumId) return;
        closeContextMenu();
        router.push(`/album/${albumId}`);
      },
    },
  ];

  return (
    <Modal
      visible={open}
      transparent
      animationType="slide"
      onRequestClose={closeContextMenu}
    >
      <Pressable
        onPress={closeContextMenu}
        className="flex-1 bg-black/55"
      >
        <Pressable
          onPress={(e) => e.stopPropagation()}
          className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7"
        >
          <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />
          <View className="flex-row items-center gap-3 px-4 py-3 border-b border-divider">
            {track.artwork ? (
              <Image
                source={track.artwork}
                className="w-12 h-12 rounded-md"
                contentFit="cover"
              />
            ) : (
              <View className="w-12 h-12 rounded-md bg-bg-card items-center justify-center">
                <Ionicons
                  name="musical-note"
                  size={20}
                  color={Colors.textMuted}
                />
              </View>
            )}
            <View className="flex-1">
              <Text
                numberOfLines={1}
                className="text-text-primary text-[15px] font-display"
              >
                {track.title}
              </Text>
              <Text
                numberOfLines={1}
                className="text-text-secondary text-xs mt-0.5 font-sans"
              >
                {track.artist}
              </Text>
            </View>
          </View>

          {items.map((item) => (
            <Pressable
              key={item.label}
              onPress={item.disabled ? undefined : item.onPress}
              android_ripple={{ color: Colors.bgHover }}
              className={`flex-row items-center gap-4 px-5 py-3.5 ${item.disabled ? "opacity-40" : ""}`}
            >
              <Ionicons
                name={item.icon}
                size={22}
                color={item.destructive ? "#FF6B6B" : Colors.textPrimary}
              />
              <Text
                className={`text-[15px] font-sans ${item.destructive ? "text-danger" : "text-text-primary"}`}
              >
                {item.label}
              </Text>
            </Pressable>
          ))}
        </Pressable>
      </Pressable>

      {/* Add-to-Playlist picker — overlays on top of the main sheet. */}
      <Modal
        visible={pickerOpen && open}
        transparent
        animationType="slide"
        onRequestClose={() => setPickerOpen(false)}
      >
        <Pressable
          onPress={() => setPickerOpen(false)}
          className="flex-1 bg-black/55"
        >
          <Pressable
            onPress={(e) => e.stopPropagation()}
            className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7"
            style={{ maxHeight: "70%" }}
          >
            <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />
            <Text className="text-text-primary text-base font-bold text-center py-2 font-sans">
              Add to playlist
            </Text>
            <ScrollView className="max-h-[60vh]">
              {playlists.length === 0 ? (
                <Text className="text-text-secondary text-sm text-center py-6 font-sans">
                  No playlists yet — create one first.
                </Text>
              ) : (
                playlists.map((p) => (
                  <Pressable
                    key={p.id}
                    onPress={() => {
                      const tid = track.id;
                      RockboxClient.addTrackToPlaylist(p.id, tid)
                        .then(() => {
                          if (process.env.EXPO_OS !== "web") {
                            Alert.alert("Added", `Added to “${p.name}”.`);
                          }
                        })
                        .catch((e: Error) => {
                          if (process.env.EXPO_OS !== "web") {
                            Alert.alert("Add failed", e.message);
                          }
                        });
                      setPickerOpen(false);
                      closeContextMenu();
                    }}
                    android_ripple={{ color: Colors.bgHover }}
                    className="flex-row items-center gap-3 px-5 py-3 active:bg-bg-hover"
                  >
                    <PlaylistCover
                      artwork={p.artwork}
                      seed={p.id || p.name}
                      size={40}
                      rounded="sm"
                      iconSize={18}
                    />
                    <View className="flex-1">
                      <Text
                        numberOfLines={1}
                        className="text-text-primary text-sm font-semibold font-sans"
                      >
                        {p.name}
                      </Text>
                      {p.isSmart ? (
                        <Text className="text-text-muted text-xs font-sans">
                          Smart playlist
                        </Text>
                      ) : null}
                    </View>
                  </Pressable>
                ))
              )}
            </ScrollView>
          </Pressable>
        </Pressable>
      </Modal>
    </Modal>
  );
}
