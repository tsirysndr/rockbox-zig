import "../global.css";
import "@/lib/nativewind-setup";
import { ThemeProvider } from "@react-navigation/native";
import { useFonts } from "expo-font";
import { Stack } from "expo-router";
import * as SplashScreen from "expo-splash-screen";
import { StatusBar } from "expo-status-bar";
import { useEffect } from "react";
import "react-native-reanimated";

import { TrackContextMenu } from "@/components/track-context-menu";
import { Colors } from "@/constants/theme";
import { PlayerProvider } from "@/lib/player-context";

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
  });

  useEffect(() => {
    if (fontsLoaded) SplashScreen.hideAsync().catch(() => {});
  }, [fontsLoaded]);

  if (!fontsLoaded) return null;

  return (
    <ThemeProvider value={navTheme}>
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
        <TrackContextMenu />
        <StatusBar style="light" />
      </PlayerProvider>
    </ThemeProvider>
  );
}
