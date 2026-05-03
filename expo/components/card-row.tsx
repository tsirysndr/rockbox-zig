import { Image } from "expo-image";
import { FlatList, Pressable, Text, View } from "react-native";

export type CardItem = {
  id: string;
  title: string;
  subtitle?: string;
  image: string;
  rounded?: "lg" | "full";
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
      renderItem={({ item }) => (
        <Pressable
          onPress={() => onPress?.(item)}
          style={({ pressed }) => ({
            width: size,
            opacity: pressed ? 0.85 : 1,
          })}
        >
          <View
            className="bg-bg-card overflow-hidden"
            style={{
              width: size,
              height: size,
              borderRadius: item.rounded === "full" ? size / 2 : 8,
              shadowColor: "#000",
              shadowOffset: { width: 0, height: 6 },
              shadowOpacity: 0.4,
              shadowRadius: 8,
            }}
          >
            <Image
              source={item.image}
              className="w-full h-full"
              contentFit="cover"
              transition={250}
            />
          </View>
          <Text
            numberOfLines={1}
            className="text-text-primary text-sm font-semibold mt-2 font-sans"
          >
            {item.title}
          </Text>
          {item.subtitle ? (
            <Text
              numberOfLines={2}
              className="text-text-secondary text-xs mt-0.5 font-sans"
            >
              {item.subtitle}
            </Text>
          ) : null}
        </Pressable>
      )}
    />
  );
}
