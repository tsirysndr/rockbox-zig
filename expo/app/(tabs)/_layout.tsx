import { Ionicons, MaterialCommunityIcons } from "@expo/vector-icons";
import type { BottomTabBarProps } from "@react-navigation/bottom-tabs";
import { Tabs } from "expo-router";
import { Pressable, Text, View } from "react-native";
import * as Haptics from "expo-haptics";

import { MiniPlayer } from "@/components/mini-player";
import { Colors } from "@/constants/theme";

type IconFamily = "ion" | "mci";

const TAB_ICONS: Record<
  string,
  { family: IconFamily; active: string; inactive: string }
> = {
  index: { family: "mci", active: "home-variant", inactive: "home-variant-outline" },
  search: { family: "ion", active: "search", inactive: "search-outline" },
  library: { family: "ion", active: "albums", inactive: "albums-outline" },
};

const TAB_LABELS: Record<string, string> = {
  index: "Home",
  search: "Search",
  library: "Your Library",
};

function TabIcon({
  family,
  name,
  size,
  color,
}: {
  family: IconFamily;
  name: string;
  size: number;
  color: string;
}) {
  if (family === "mci") {
    return (
      <MaterialCommunityIcons name={name as any} size={size} color={color} />
    );
  }
  return <Ionicons name={name as any} size={size} color={color} />;
}

function CustomTabBar({ state, navigation }: BottomTabBarProps) {
  return (
    <View className="bg-bg">
      <View
        className="bg-bg-dock rounded-t-[20px] overflow-hidden"
        style={{
          shadowColor: "#000",
          shadowOpacity: 0.5,
          shadowRadius: 12,
          shadowOffset: { width: 0, height: -6 },
        }}
      >
        <MiniPlayer />
        <View className="flex-row pt-1.5 pb-6 px-2">
          {state.routes.map((route, idx) => {
            const isFocused = state.index === idx;
            const iconConfig = TAB_ICONS[route.name];
            const label = TAB_LABELS[route.name] ?? route.name;
            if (!iconConfig) return null;

            return (
              <Pressable
                key={route.key}
                accessibilityRole="button"
                onPress={() => {
                  if (process.env.EXPO_OS === "ios") {
                    Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
                  }
                  const event = navigation.emit({
                    type: "tabPress",
                    target: route.key,
                    canPreventDefault: true,
                  });
                  if (!isFocused && !event.defaultPrevented) {
                    navigation.navigate(route.name, route.params);
                  }
                }}
                className="flex-1 items-center justify-center gap-0.5 py-1"
              >
                <TabIcon
                  family={iconConfig.family}
                  name={isFocused ? iconConfig.active : iconConfig.inactive}
                  size={26}
                  color={isFocused ? Colors.textPrimary : Colors.textMuted}
                />
                <Text
                  className={`text-[11px] font-medium ${isFocused ? "text-text-primary" : "text-text-muted"}`}
                >
                  {label}
                </Text>
              </Pressable>
            );
          })}
        </View>
      </View>
    </View>
  );
}

export default function TabLayout() {
  return (
    <Tabs
      tabBar={(props) => <CustomTabBar {...props} />}
      screenOptions={{ headerShown: false }}
    >
      <Tabs.Screen name="index" options={{ title: "Home" }} />
      <Tabs.Screen name="search" options={{ title: "Search" }} />
      <Tabs.Screen name="library" options={{ title: "Your Library" }} />
    </Tabs>
  );
}
