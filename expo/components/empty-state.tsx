import { Ionicons } from "@expo/vector-icons";
import { router } from "expo-router";
import { Pressable, Text, View } from "react-native";

import { Colors } from "@/constants/theme";

/**
 * Standard empty-state for screens that need a connected server but don't
 * have one. Shows an icon, a short message, and a "Choose server" CTA.
 */
export function NotConnectedState({
  message = "Connect to a rockbox server to see your library.",
  icon = "server-outline",
}: {
  message?: string;
  icon?: React.ComponentProps<typeof Ionicons>["name"];
}) {
  return (
    <View className="px-6 pt-12 items-center">
      <Ionicons name={icon} size={48} color={Colors.textMuted} />
      <Text className="text-text-secondary text-[14px] text-center mt-4 font-sans">
        {message}
      </Text>
      <Pressable
        onPress={() => router.push("/settings/server")}
        className="mt-5 px-4 py-2 rounded-full bg-accent active:opacity-80"
      >
        <Text className="text-white text-[13px] font-bold font-sans">
          Choose server
        </Text>
      </Pressable>
    </View>
  );
}
