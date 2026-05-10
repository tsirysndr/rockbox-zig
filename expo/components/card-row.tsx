import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { FlatList, Pressable, Text, View } from "react-native";

import { gradientColors } from "@/components/playlist-cover";
import { Colors } from "@/constants/theme";

export type CardItem = {
  id: string;
  title: string;
  subtitle?: string;
  image: string;
  rounded?: "lg" | "full";
  /** Icon shown in the placeholder when `image` is empty. */
  placeholderIcon?: React.ComponentProps<typeof Ionicons>["name"];
  /** When true and `image` is empty, render a colorful gradient backdrop. */
  colorfulPlaceholder?: boolean;
};

export function CardRow({
  data,
  onPress,
  size = 160,
}: {
  data: CardItem[];
  onPress?: (item: CardItem) => void;
  size?: number;
}) {
  return (
    <FlatList
      data={data}
      horizontal
      keyExtractor={(item) => item.id}
      showsHorizontalScrollIndicator={false}
      contentContainerStyle={{ paddingHorizontal: 16, gap: 14 }}
      renderItem={({ item }) => {
        const round = item.rounded === "full";
        return (
          <Pressable
            onPress={() => onPress?.(item)}
            style={{ width: size }}
            className={`active:opacity-80 ${round ? "items-center" : ""}`}
          >
            <View
              className="bg-bg-card overflow-hidden items-center justify-center"
              style={{
                width: size,
                height: size,
                borderRadius: round ? size / 2 : 8,
                shadowColor: "#000",
                shadowOffset: { width: 0, height: 6 },
                shadowOpacity: 0.4,
                shadowRadius: 8,
              }}
            >
              {item.image ? (
                <Image
                  source={item.image}
                  className="w-full h-full"
                  contentFit="cover"
                  transition={250}
                />
              ) : item.colorfulPlaceholder ? (
                <LinearGradient
                  colors={gradientColors(item.id || item.title)}
                  start={{ x: 0, y: 0 }}
                  end={{ x: 1, y: 1 }}
                  style={{
                    width: "100%",
                    height: "100%",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  <Ionicons
                    name={item.placeholderIcon ?? "musical-notes"}
                    size={Math.round(size * 0.35)}
                    color="#FFFFFF"
                  />
                </LinearGradient>
              ) : (
                <Ionicons
                  name={item.placeholderIcon ?? "musical-notes"}
                  size={Math.round(size * 0.35)}
                  color={Colors.textMuted}
                />
              )}
            </View>
            <Text
              numberOfLines={1}
              ellipsizeMode="tail"
              className={`text-text-primary text-sm font-semibold mt-2 font-sans ${round ? "text-center" : ""}`}
            >
              {item.title}
            </Text>
            {item.subtitle ? (
              <Text
                numberOfLines={1}
                ellipsizeMode="tail"
                className={`text-text-secondary text-xs mt-0.5 font-sans ${round ? "text-center" : ""}`}
              >
                {item.subtitle}
              </Text>
            ) : null}
          </Pressable>
        );
      }}
    />
  );
}
