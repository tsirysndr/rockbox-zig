import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { Modal, Pressable, Text, View } from "react-native";

import { Colors } from "@/constants/theme";

export type ActionItem = {
  icon: React.ComponentProps<typeof Ionicons>["name"];
  label: string;
  onPress?: () => void;
  destructive?: boolean;
  disabled?: boolean;
};

type Header = {
  title: string;
  subtitle?: string;
  image?: string;
  rounded?: "lg" | "full";
};

export function ActionSheet({
  visible,
  onClose,
  header,
  actions,
}: {
  visible: boolean;
  onClose: () => void;
  header?: Header;
  actions: ActionItem[];
}) {
  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <Pressable
        onPress={onClose}
        className="flex-1 bg-black/55"
      >
        <Pressable
          onPress={(e) => e.stopPropagation()}
          className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7"
        >
          <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />

          {header ? (
            <View className="flex-row items-center gap-3 px-4 py-3 border-b border-divider">
              {header.image ? (
                <Image
                  source={header.image}
                  className={`w-12 h-12 ${header.rounded === "full" ? "rounded-full" : "rounded-md"}`}
                  contentFit="cover"
                />
              ) : (
                <View
                  className={`w-12 h-12 bg-bg-card items-center justify-center ${header.rounded === "full" ? "rounded-full" : "rounded-md"}`}
                >
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
                  {header.title}
                </Text>
                {header.subtitle ? (
                  <Text
                    numberOfLines={1}
                    className="text-text-secondary text-xs mt-0.5 font-sans"
                  >
                    {header.subtitle}
                  </Text>
                ) : null}
              </View>
            </View>
          ) : null}

          {actions.map((item) => (
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
