import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { router } from "expo-router";
import { Modal, Pressable, Text, View } from "react-native";

import { Colors } from "@/constants/theme";
import { ALBUMS, ARTISTS } from "@/lib/mock-data";
import { usePlayer } from "@/lib/player-context";

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
  const album = ALBUMS.find((a) => a.title === track.album);
  const artist = ARTISTS.find((a) => a.name === track.artist);

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
        closeContextMenu();
      },
    },
    {
      icon: "person-outline",
      label: "Go to Artist",
      disabled: !artist,
      onPress: () => {
        if (!artist) return;
        closeContextMenu();
        router.push(`/artist/${artist.id}`);
      },
    },
    {
      icon: "disc-outline",
      label: "Go to Album",
      disabled: !album,
      onPress: () => {
        if (!album) return;
        closeContextMenu();
        router.push(`/album/${album.id}`);
      },
    },
    {
      icon: "share-outline",
      label: "Share",
      onPress: closeContextMenu,
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
                className="text-text-primary text-[15px] font-bold font-sans"
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
    </Modal>
  );
}
