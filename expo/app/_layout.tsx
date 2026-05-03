import "../global.css";
import "@/lib/nativewind-setup";
import { ThemeProvider } from "@react-navigation/native";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useFonts } from "expo-font";
import { Stack } from "expo-router";
import * as SplashScreen from "expo-splash-screen";
import { StatusBar } from "expo-status-bar";
import { useEffect, useState } from "react";
import "react-native-reanimated";

import { PersistentMiniPlayer } from "@/components/persistent-mini-player";
import { TrackContextMenu } from "@/components/track-context-menu";
import { Colors } from "@/constants/theme";
import { PlayerProvider } from "@/lib/player-context";
import { RockboxStreams } from "@/lib/rockbox-streams";

SplashScreen.preventAutoHideAsync().catch(() => {});

const navTheme = {
  dark: true,
  colors: {
    primary: Colors.accent,
    background: Colors.appBg,
    card: Colors.appBg,
    text: Colors.textPrimary,
    border: Colors.divider,
    notification: Colors.accent,
  },
  fonts: {
    regular: { fontFamily: "SpaceGrotesk", fontWeight: "400" as const },
    medium: { fontFamily: "SpaceGrotesk", fontWeight: "500" as const },
    bold: { fontFamily: "SpaceGrotesk", fontWeight: "700" as const },
    heavy: { fontFamily: "SpaceGrotesk", fontWeight: "800" as const },
  },
};

export const unstable_settings = {
  anchor: "(tabs)",
};

export default function RootLayout() {
  const [fontsLoaded] = useFonts({
    SpaceGrotesk: require("@/assets/fonts/SpaceGrotesk.ttf"),
    JetBrainsMono: require("@/assets/fonts/JetBrainsMono.ttf"),
    "RockfordSans-Light": require("@/assets/fonts/RockfordSans-Light.otf"),
    "RockfordSans-Regular": require("@/assets/fonts/RockfordSans-Regular.otf"),
    "RockfordSans-RegularItalic": require("@/assets/fonts/RockfordSans-RegularItalic.otf"),
    "RockfordSans-Medium": require("@/assets/fonts/RockfordSans-Medium.otf"),
    "RockfordSans-Bold": require("@/assets/fonts/RockfordSans-Bold.otf"),
    "RockfordSans-BoldItalic": require("@/assets/fonts/RockfordSans-BoldItalic.otf"),
    "RockfordSans-ExtraBold": require("@/assets/fonts/RockfordSans-ExtraBold.otf"),
  });
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 5 * 60 * 1000,
            retry: 1,
          },
        },
      }),
  );

  useEffect(() => {
    if (fontsLoaded) SplashScreen.hideAsync().catch(() => {});
  }, [fontsLoaded]);

  if (!fontsLoaded) return null;

  return (
    <ThemeProvider value={navTheme}>
      <QueryClientProvider client={queryClient}>
      <RockboxStreams />
      <PlayerProvider>
        <Stack
          screenOptions={{
            headerShown: false,
            contentStyle: { backgroundColor: Colors.appBg },
          }}
        >
          <Stack.Screen name="(tabs)" />
          <Stack.Screen
            name="player"
            options={{
              presentation: "modal",
              animation: "slide_from_bottom",
            }}
          />
          <Stack.Screen
            name="queue"
            options={{
              presentation: "modal",
              animation: "slide_from_bottom",
            }}
          />
          <Stack.Screen
            name="settings"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="settings/server"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="settings/bluetooth"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="album/[id]"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="artist/[id]"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="playlist/[id]"
            options={{ animation: "slide_from_right" }}
          />
          <Stack.Screen
            name="playlist/new"
            options={{
              presentation: "modal",
              animation: "slide_from_bottom",
            }}
          />
          <Stack.Screen
            name="genre/[id]"
            options={{ animation: "slide_from_right" }}
          />
        </Stack>
        <PersistentMiniPlayer />
        <TrackContextMenu />
        <StatusBar style="light" />
      </PlayerProvider>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
